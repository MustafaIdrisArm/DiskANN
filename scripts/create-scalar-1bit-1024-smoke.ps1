param(
    [string]$RepositoryRoot = (Split-Path -Parent $PSScriptRoot)
)

$ErrorActionPreference = "Stop"

$root = (Resolve-Path -LiteralPath $RepositoryRoot).Path
$dataDirectory = Join-Path $root "test_data/disk_index_search"
$exampleDirectory = Join-Path $root "diskann-benchmark/example"
$converter = Join-Path $PSScriptRoot "expand-fbin-dimensions.ps1"

$sourceData = Join-Path $dataDirectory "disk_index_siftsmall_learn_256pts_data.fbin"
$sourceQueries = Join-Path $dataDirectory "disk_index_sample_query_10pts.fbin"
$outputDataName = "disk_index_siftsmall_learn_256pts_data_1024d.fbin"
$outputQueriesName = "disk_index_sample_query_10pts_1024d.fbin"
$outputData = Join-Path $dataDirectory $outputDataName
$outputQueries = Join-Path $dataDirectory $outputQueriesName

& $converter -InputPath $sourceData -OutputPath $outputData -Repeat 8
& $converter -InputPath $sourceQueries -OutputPath $outputQueries -Repeat 8

$sourceConfig = Join-Path $exampleDirectory "scalar-1bit-smoke.json"
$outputConfig = Join-Path $exampleDirectory "scalar-1bit-1024-smoke.json"
$config = Get-Content -LiteralPath $sourceConfig -Raw | ConvertFrom-Json
$config.jobs[0].content.build.data = $outputDataName
$config.jobs[0].content.search_phase.queries = $outputQueriesName
$config | ConvertTo-Json -Depth 32 | Set-Content -LiteralPath $outputConfig -Encoding utf8

Write-Host "Created $outputConfig."
Write-Host "The existing squared-L2 ground truth remains valid because every coordinate is repeated eight times."
