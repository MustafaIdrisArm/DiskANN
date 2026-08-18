param(
    [Parameter(Mandatory = $true)]
    [string]$InputPath,

    [Parameter(Mandatory = $true)]
    [string]$OutputPath,

    [Parameter(Mandatory = $true)]
    [ValidateRange(1, [int]::MaxValue)]
    [int]$Repeat
)

$ErrorActionPreference = "Stop"

$inputFile = (Resolve-Path -LiteralPath $InputPath).Path
$outputFile = [System.IO.Path]::GetFullPath($OutputPath)

if ($inputFile -eq $outputFile) {
    throw "InputPath and OutputPath must be different."
}

$reader = [System.IO.BinaryReader]::new([System.IO.File]::OpenRead($inputFile))
try {
    $rows = $reader.ReadUInt32()
    $inputDim = $reader.ReadUInt32()
    $outputDim = [uint64]$inputDim * [uint64]$Repeat
    if ($outputDim -gt [uint32]::MaxValue) {
        throw "Expanded dimension $outputDim does not fit in an fbin header."
    }

    $expectedLength = 8L + 4L * [int64]$rows * [int64]$inputDim
    if ($reader.BaseStream.Length -ne $expectedLength) {
        throw "Invalid fbin size: expected $expectedLength bytes, found $($reader.BaseStream.Length)."
    }

    $outputDirectory = [System.IO.Path]::GetDirectoryName($outputFile)
    if ($outputDirectory) {
        [System.IO.Directory]::CreateDirectory($outputDirectory) | Out-Null
    }

    $writer = [System.IO.BinaryWriter]::new([System.IO.File]::Create($outputFile))
    try {
        $writer.Write($rows)
        $writer.Write([uint32]$outputDim)

        $rowBytes = [int]$inputDim * 4
        for ($row = 0; $row -lt $rows; $row++) {
            $values = $reader.ReadBytes($rowBytes)
            if ($values.Length -ne $rowBytes) {
                throw "Unexpected end of file while reading row $row."
            }
            for ($copy = 0; $copy -lt $Repeat; $copy++) {
                $writer.Write($values)
            }
        }
    }
    finally {
        $writer.Dispose()
    }
}
finally {
    $reader.Dispose()
}

Write-Host "Created $outputFile ($rows vectors, $inputDim -> $outputDim dimensions)."
