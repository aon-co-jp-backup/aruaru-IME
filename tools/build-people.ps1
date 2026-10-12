# 人名・有名人名・重要人物名の辞書表(crates/aruaru-ime-core/pack_tables/people.tsv)を、Wikidata(CC0)から作る。
# 職業ごとに「リンクされている言語版の数(sitelinks)」が多い人物を取り、日本語の読みが分かるものだけを入れる。
#   - 日本語表記が かな/カタカナ だけ: その表記から読みを作る(中黒・空白は除く)
#   - 日本語表記に漢字を含む: Wikidata の「かな表記(P1814)」があるものだけ
#   - ラテン文字を含む表記(例: ジョージ・W・ブッシュ)は、読みが定まらないので除く
# 列: よみ / 日本語 / English / 簡体 / 繁体 / ピンイン / 한글 / fr / es / de / it / pt(簡体・繁体・ピンインは空)
# 使い方: powershell -ExecutionPolicy Bypass -File tools\build-people.ps1 [-MinSitelinks 90] [-PerOccupation 300]
param([int]$MinSitelinks = 90, [int]$PerOccupation = 300, [switch]$Japan)
if ($Japan -and -not $PSBoundParameters.ContainsKey("MinSitelinks")) { $MinSitelinks = 25 }
if ($Japan -and -not $PSBoundParameters.ContainsKey("PerOccupation")) { $PerOccupation = 200 }
$ErrorActionPreference = "Stop"
$out = Join-Path $PSScriptRoot "..\crates\aruaru-ime-core\pack_tables\people.tsv"
$ua = "aruaru-IME-dict-builder/0.1 (https://github.com/aon-co-jp/aruaru-IME)"

$occ = @(
  "Q82955","Q372436","Q116","Q48352","Q189290","Q3242115",
  "Q901","Q169470","Q170790","Q593644","Q864503","Q11063","Q205375","Q81096","Q39631","Q188094",
  "Q4964182","Q201788","Q1234713",
  "Q36180","Q49757","Q6625963","Q214917","Q1930187",
  "Q1028181","Q1281618","Q42973","Q33231",
  "Q36834","Q177220","Q486748","Q158852","Q639669",
  "Q33999","Q2526255","Q28389","Q191633","Q266569","Q245068","Q622807",
  "Q2066131","Q937857","Q10871364","Q10833314","Q3665646","Q11303721","Q11338576","Q378622","Q10873124",
  "Q43845","Q131524","Q11900058","Q11631"
)

function Get-Lit($v) {
  if ($null -eq $v -or $v -eq "") { return "" }
  $m = [regex]::Match($v, '^"(.*)"(@[A-Za-z-]+)?$')
  if ($m.Success) { return $m.Groups[1].Value }
  return $v
}
function To-Hira([string]$s) {
  $sb = New-Object System.Text.StringBuilder
  foreach ($c in $s.ToCharArray()) {
    $n = [int]$c
    if ($n -ge 0x30A1 -and $n -le 0x30F6) { [void]$sb.Append([char]($n - 0x60)) } else { [void]$sb.Append($c) }
  }
  $sb.ToString()
}
# かな(ひらがな・カタカナ・長音)と、除く記号(中黒・空白)だけでできているか
function Kana-Only([string]$s) { $s -match '^[ぁ-ゖァ-ヺー・･·\s=＝]+$' }
function Reading([string]$ja, [string]$kana) {
  if (Kana-Only $ja) { $r = $ja } elseif ($kana -and (Kana-Only $kana)) { $r = $kana } else { return "" }
  $r = To-Hira $r
  ($r -replace '[・･·\s=＝]', '')
}

$seen = @{}
$seenJa = @{}
$rows = New-Object System.Collections.Generic.List[string]
# -Japan: 既存の表に、日本の人物(国籍または出生地が日本)を追加する(世界の上位だけでは日本人が上位300件に入らないため)
if ($Japan -and (Test-Path $out)) { foreach ($l in [IO.File]::ReadAllLines($out, [Text.Encoding]::UTF8)) { if ($l) { $rows.Add($l); $seenJa[($l -split "`t")[1]] = 1 } } }
$jpFilter = if ($Japan) { "{ ?p wdt:P27 wd:Q17 } UNION { ?p wdt:P19 ?pl . ?pl wdt:P17 wd:Q17 }" } else { "" }
$langs = @("ko","fr","es","de","it","pt")
foreach ($q in $occ) {
  $opt = ($langs | ForEach-Object { "OPTIONAL { ?p rdfs:label ?$_ . FILTER(LANG(?$_)=`"$_`") }" }) -join "`n"
  $query = @"
SELECT ?p ?ja ?en ?kana ?ko ?fr ?es ?de ?it ?pt ?sl ?jp WHERE {
  ?p wdt:P31 wd:Q5; wdt:P106 wd:$q; wikibase:sitelinks ?sl. FILTER(?sl >= $MinSitelinks)
  ?p rdfs:label ?ja. FILTER(LANG(?ja)="ja")
  ?p rdfs:label ?en. FILTER(LANG(?en)="en")
  OPTIONAL { ?p wdt:P1814 ?kana }
  $jpFilter
  BIND(EXISTS { ?p wdt:P27 wd:Q17 } AS ?jp)
  $opt
} ORDER BY DESC(?sl) LIMIT $PerOccupation
"@
  $tmp = [IO.Path]::GetTempFileName()
  [IO.File]::WriteAllText($tmp, $query, (New-Object System.Text.UTF8Encoding($false)))
  $res = $null
  for ($try = 0; $try -lt 3 -and -not $res; $try++) {
    try {
      $o = "$tmp.out"
      & curl.exe -s -m 120 -G "https://query.wikidata.org/sparql" --data-urlencode "query@$tmp" -H "Accept: text/tab-separated-values" -A $ua -o $o
      $res = [IO.File]::ReadAllText($o, [Text.Encoding]::UTF8)
      Remove-Item $o -ErrorAction SilentlyContinue
      if ($LASTEXITCODE -ne 0 -or $res -match "timeout|<html|^s*$") { $res = $null; Start-Sleep 5 }
    } catch { Start-Sleep 5 }
  }
  Remove-Item $tmp -ErrorAction SilentlyContinue
  if (-not $res) { Write-Host "skip $q (取得失敗)"; continue }
  $lines = $res -split "`n"
  $n = 0
  foreach ($l in ($lines | Select-Object -Skip 1)) {
    $c = $l -split "`t"
    if ($c.Count -lt 12) { continue }
    $id = $c[0]
    if ($seen.ContainsKey($id)) { continue }
    if ($seenJa.ContainsKey((Get-Lit $c[1]))) { continue }
    $ja = Get-Lit $c[1]; $en = Get-Lit $c[2]; $kana = Get-Lit $c[3]
    if ($ja -match '[A-Za-z0-9]') { continue }
    $r = Reading $ja $kana
    # 日本国籍で、読みが無い漢字表記: 英語表記(ヘボン式)から読みを作る(先頭の = は「ローマ字から変換」の印。IME側の romaji で変換する)
    if (-not $r -and ($c[11] -match 'true' -or $Japan)) {
      $rom = $en.ToLowerInvariant().Normalize([Text.NormalizationForm]::FormKC)
      foreach ($p in @(@([string][char]0x14D, 'ou'), @([string][char]0x16B, 'uu'), @([string][char]0x101, 'aa'), @([string][char]0x12B, 'ii'), @([string][char]0x113, 'ee'))) { $rom = $rom.Replace($p[0], $p[1]) }
      $rom = ($rom -replace "[^a-z']", '')
      if ($rom.Length -ge 3 -and $en -match '^[A-Za-zĀ-ſ\s''.-]+$') { $r = "=$rom" }
    }
    if (-not $r) { continue }
    $seen[$id] = 1
    $ko = Get-Lit $c[4]; $fr = Get-Lit $c[5]; $es = Get-Lit $c[6]; $de = Get-Lit $c[7]; $it = Get-Lit $c[8]; $pt = Get-Lit $c[9]
    $rows.Add(($r, $ja, $en, "", "", "", $ko, $fr, $es, $de, $it, $pt) -join "`t")
    $n++
  }
  Write-Host "$q : +$n (計 $($rows.Count))"
  Start-Sleep 2
}
[IO.File]::WriteAllText($out, (($rows | Sort-Object -Unique) -join "`n") + "`n", (New-Object System.Text.UTF8Encoding($false)))
Write-Host "書き出し: $out ($($rows.Count) 件)"
