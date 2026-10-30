param(
  [string]$BaseUrl = "http://127.0.0.1:8080",
  [switch]$StartServer,
  [switch]$SkipStartServer
)

$ErrorActionPreference = "Stop"

function Write-Step([string]$msg) {
  Write-Host "[SMOKE] $msg"
}

function Invoke-ApiJson {
  param(
    [string]$Method,
    [string]$Url,
    [object]$Body,
    [int[]]$ExpectedStatus = @(200)
  )

  $json = if ($null -ne $Body) { $Body | ConvertTo-Json -Depth 30 } else { $null }

  try {
    if ($null -ne $json) {
      $resp = Invoke-WebRequest -Method $Method -Uri $Url -ContentType "application/json" -Body $json
    } else {
      $resp = Invoke-WebRequest -Method $Method -Uri $Url
    }
    $status = [int]$resp.StatusCode
    if ($ExpectedStatus -notcontains $status) {
      throw "Status non atteso $status su $Url"
    }
    if ([string]::IsNullOrWhiteSpace($resp.Content)) {
      return $null
    }
    return $resp.Content | ConvertFrom-Json
  } catch {
    if ($_.Exception.Response -ne $null) {
      $status = [int]$_.Exception.Response.StatusCode
      if ($ExpectedStatus -contains $status) {
        $reader = New-Object System.IO.StreamReader($_.Exception.Response.GetResponseStream())
        $content = $reader.ReadToEnd()
        if ([string]::IsNullOrWhiteSpace($content)) {
          return $null
        }
        return $content | ConvertFrom-Json
      }
    }
    throw
  }
}

$repoRoot = Split-Path -Parent $PSScriptRoot
$sampleIliPath = Join-Path $repoRoot "..\testdata\canalizzazioni_ut4_v1_0.ili"
if (-not (Test-Path $sampleIliPath)) {
  throw "File test non trovato: $sampleIliPath"
}

$serverProcess = $null
try {
  if (-not $SkipStartServer -and $StartServer) {
    Write-Step "Avvio server locale con cargo run"
    $cargo = Get-Command cargo -ErrorAction SilentlyContinue
    if ($null -eq $cargo) {
      throw "cargo non trovato. Avvia il server manualmente e rilancia con -SkipStartServer"
    }

    $serverProcess = Start-Process -FilePath $cargo.Source -ArgumentList "run" -WorkingDirectory $repoRoot -PassThru

    Write-Step "Attendo endpoint /health"
    $ok = $false
    for ($i = 0; $i -lt 40; $i++) {
      Start-Sleep -Milliseconds 500
      try {
        $health = Invoke-ApiJson -Method GET -Url "$BaseUrl/health" -Body $null -ExpectedStatus @(200)
        if ($null -ne $health) {
          $ok = $true
          break
        }
      } catch {
      }
    }
    if (-not $ok) {
      throw "Server non raggiungibile su $BaseUrl/health"
    }
  } else {
    Write-Step "Uso server gia avviato"
    [void](Invoke-ApiJson -Method GET -Url "$BaseUrl/health" -Body $null -ExpectedStatus @(200))
  }

  Write-Step "Import caso valido (.ili reale)"
  $iliText = Get-Content -Raw -Encoding UTF8 $sampleIliPath
  $importResp = Invoke-ApiJson -Method POST -Url "$BaseUrl/api/model/import/ili" -Body @{ interlis = $iliText } -ExpectedStatus @(200)
  if ($null -eq $importResp.model) {
    throw "Import non ha restituito model"
  }

  Write-Step "Validate modello importato"
  $validateResp = Invoke-ApiJson -Method POST -Url "$BaseUrl/api/model/validate" -Body $importResp.model -ExpectedStatus @(200)
  if ($null -eq $validateResp.valid) {
    throw "Validate non ha restituito flag valid"
  }

  Write-Step "Export .ili del modello"
  $exportResp = Invoke-ApiJson -Method POST -Url "$BaseUrl/api/model/export/ili" -Body $importResp.model -ExpectedStatus @(200)
  if ([string]::IsNullOrWhiteSpace($exportResp.interlis)) {
    throw "Export non ha restituito testo INTERLIS"
  }

  Write-Step "Round-trip check metadati MODEL"
  $reimportResp = Invoke-ApiJson -Method POST -Url "$BaseUrl/api/model/import/ili" -Body @{ interlis = $exportResp.interlis } -ExpectedStatus @(200)
  if ($reimportResp.model.model_name -ne $importResp.model.model_name) {
    throw "Round-trip model_name differente"
  }
  if ($reimportResp.model.model_at -ne $importResp.model.model_at) {
    throw "Round-trip MODEL AT differente"
  }
  if ($reimportResp.model.model_model_version -ne $importResp.model.model_model_version) {
    throw "Round-trip MODEL VERSION differente"
  }

  Write-Step "Caso negativo: MODEL malformato (atteso 422)"
  $badIli = @"
INTERLIS 2.4;
MODEL Broken (de) AT ""http://example.org"" VERSION ""1.0"" =
  TOPIC Main =
    CLASS A =
      attr: TEXT;
    END A;
  END Main;
END Broken
"@
  $badResp = Invoke-ApiJson -Method POST -Url "$BaseUrl/api/model/import/ili" -Body @{ interlis = $badIli } -ExpectedStatus @(422)
  if ($null -eq $badResp.issues -or $badResp.issues.Count -eq 0) {
    throw "Caso negativo senza issues"
  }

  Write-Step "OK: smoke test completato"
  exit 0
}
finally {
  if ($null -ne $serverProcess -and -not $serverProcess.HasExited) {
    Write-Step "Arresto server avviato dallo script"
    Stop-Process -Id $serverProcess.Id -Force
  }
}
