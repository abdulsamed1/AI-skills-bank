$sourcePath = "c:\Users\ASUS\production\skill-manage\src\source.md"
$content = Get-Content $sourcePath -Raw

$matches = [regex]::Matches($content, 'https://github\.com/([^/]+)/([^/)\s\n''"]+)')

$urlsToClone = @{}
foreach ($match in $matches) {
    if ($match.Groups.Count -gt 2) {
        $owner = $match.Groups[1].Value
        $repo = $match.Groups[2].Value -replace '\.git$', '' -replace '/tree/.*$', '' -replace '/$', ''
        $cleanUrl = "https://github.com/$owner/$repo"
        $urlsToClone[$cleanUrl] = "$owner-$repo"
    }
}

$existingUrls = @{}
$subdirs = Get-ChildItem "c:\Users\ASUS\production\skill-manage\src" -Directory
foreach ($dir in $subdirs) {
    if (Test-Path "$($dir.FullName)\.git") {
        $remoteUrl = git -C $dir.FullName remote get-url origin 2>$null
        if ($remoteUrl) {
            $cleanRemoteUrl = $remoteUrl.Trim() -replace '\.git$', ''
            $existingUrls[$cleanRemoteUrl] = $true
        }
    }
}

Write-Host "URLs to ensure: $($urlsToClone.Count)"
Write-Host "Existing repos: $($existingUrls.Count)"

$cloned = 0
foreach ($url in $urlsToClone.Keys) {
    if (-not $existingUrls.ContainsKey($url)) {
        $targetDir = $urlsToClone[$url]
        # Make sure folder doesn't exist
        if (-not (Test-Path "c:\Users\ASUS\production\skill-manage\src\$targetDir")) {
            Write-Host "Cloning $url into $targetDir"
            $gitUrl = "$url.git"
            git clone $gitUrl "c:\Users\ASUS\production\skill-manage\src\$targetDir"
            $cloned++
        } else {
            Write-Host "Target directory $targetDir exists but no git remote found for $url. Skipping."
        }
    }
}
Write-Host "Finished cloning $cloned new repos."
