#!/usr/bin/env node
import { Command } from "commander";
import chalk from "chalk";
import boxen from "boxen";
import ora from "ora";
import { select, input, confirm, checkbox } from "@inquirer/prompts";
import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { mkdirSync } from "node:fs";
import { dirname, join, resolve } from "node:path";

const program = new Command();
const PROC_TIMEOUT_MS = 10 * 60 * 1000;
const EXCLUSION_CATEGORY_CHOICES = [
  "games",
  "law-legal",
  "medicine-medical",
  "pharmacy",
  "biology",
  "chemistry",
  "llm-from-scratch"
];

function sanitizeOutput(value) {
  if (!value) return "";
  return String(value).replace(/\r\n/g, "\n").trim();
}

function isValidRepoUrl(repoUrl) {
  if (!repoUrl || typeof repoUrl !== "string") return false;
  const trimmed = repoUrl.trim();
  if (!trimmed || /[\r\n\t]/.test(trimmed) || /["'`]/.test(trimmed)) {
    return false;
  }

  return /^https:\/\/.+/.test(trimmed) || /^git@[^:]+:.+/.test(trimmed);
}

function isSafeFolderName(folderName) {
  if (!folderName) return true;
  if (folderName.length > 100) return false;
  if (folderName === "." || folderName === "..") return false;
  if (folderName.includes("/") || folderName.includes("\\")) return false;
  return /^[a-zA-Z0-9._-]+$/.test(folderName);
}

function printBanner() {
  const title = chalk.bold.hex("#00d4ff")("AI Skills Bank CLI");
  const subtitle = chalk.hex("#7dd3fc")("Secure skill routing with visual workflows");
  const rails = chalk.hex("#22d3ee")(">>>>>");
  const content = `${rails} ${title} ${rails}\n${subtitle}`;
  console.log(
    boxen(content, {
      padding: 1,
      borderColor: "blue",
      borderStyle: "double"
    })
  );
}

function getOutputPreferences() {
  const opts = program.opts();
  return {
    verbose: Boolean(opts.verbose),
    rawOutput: Boolean(opts.rawOutput)
  };
}

function toReadableOutput(label, stdout, { verbose, rawOutput }) {
  if (!stdout) return "";
  if (verbose || rawOutput) return stdout;

  const lines = stdout.split("\n").map((x) => x.trimEnd());
  const normalize = (line) =>
    line
      .replace(/^\[\d{2}:\d{2}:\d{2}\]\s*/i, "")
      .replace(/^\[(INFO|WARN|ERROR)\]\s*/i, "")
      .trim();
  const keepPatterns = [
    /^src repo mode:/i,
    /^Multi-hub mode:/i,
    /Loaded \d+ skills/i,
    /Excluded skills by policy/i,
    /Categorized into \d+ sub-hubs/i,
    /Aggregation Complete/i,
    /^Sub-hubs created:/i,
    /^Total skills:/i,
    /^Output dir:/i,
    /^Starting Hub Sync/i,
    /^Sync mode:/i,
    /^Found \d+ hubs to sync/i,
    /^Synced:\s+\d+\/\d+\s+hubs/i,
    /^Modes:/i,
    /^Summary:/i,
    /^Hubs synced:/i,
    /^Targets:/i,
    /^Synced to:/i,
    /Security audit/i,
    /ERROR|WARN/i,
    /^\[✓\]\s+Loaded/i,
    /^\[✓\]\s+Categorized/i
  ];

  const noisePatterns = [
    /^\[INFO\]\s+\s*✓\s+.*\[(Junction|Copy|SymbolicLink)\]/i,
    /^\[✓\]\s+[^:]+:\s+\d+\s+skills.*router mode$/i,
    /^\[INFO\]\s*=+/i,
    /^\[INFO\]\s*$/i
  ];

  const kept = [];
  for (const line of lines) {
    const trimmed = line.trim();
    const canonical = normalize(trimmed);
    if (!trimmed) continue;
    if (noisePatterns.some((r) => r.test(trimmed) || r.test(canonical))) continue;
    if (keepPatterns.some((r) => r.test(trimmed) || r.test(canonical))) {
      kept.push(canonical);
    }
  }

  if (kept.length === 0) {
    return lines.slice(0, 30).join("\n");
  }

  if (label.toLowerCase() === "sync") {
    const limited = [];
    let seenSynced = false;
    let seenModes = false;
    for (const line of kept) {
      if (/^Synced:\s+\d+\/\d+\s+hubs/i.test(line)) {
        if (seenSynced) continue;
        seenSynced = true;
      }
      if (/^Modes:/i.test(line)) {
        if (seenModes) continue;
        seenModes = true;
      }
      limited.push(line);
    }

    const syncedTargetsIndex = kept.findIndex((x) => /Synced to:/i.test(x));
    if (syncedTargetsIndex >= 0) {
      const head = limited.slice(0, syncedTargetsIndex + 1);
      const tail = limited.slice(syncedTargetsIndex + 1);
      const dedupTail = [...new Set(tail)];
      return [...head, ...dedupTail].join("\n");
    }
    return [...new Set(limited)].join("\n");
  }

  return [...new Set(kept)].join("\n");
}

function runCmd(command, args, cwd, timeoutMs = PROC_TIMEOUT_MS) {
  const result = spawnSync(command, args, {
    cwd,
    stdio: "pipe",
    encoding: "utf8",
    shell: false,
    timeout: timeoutMs,
    windowsHide: true
  });

  return {
    code: result.status ?? 1,
    stdout: sanitizeOutput(result.stdout),
    stderr: sanitizeOutput(result.stderr),
    error: result.error
  };
}

function runPowerShellScript(scriptPath, scriptArgs, cwd) {
  const shells = ["pwsh", "powershell"];
  for (const shell of shells) {
    const probe = runCmd(shell, ["-NoProfile", "-Command", "$PSVersionTable.PSVersion.ToString()"], cwd);
    if (probe.code !== 0) {
      continue;
    }

    const args = ["-ExecutionPolicy", "Bypass", "-File", scriptPath, ...scriptArgs];
    return runCmd(shell, args, cwd);
  }

  return {
    code: 1,
    stdout: "",
    stderr: "No PowerShell runtime found (expected pwsh or powershell)."
  };
}

function resolveProjectPaths(projectRootInput) {
  const projectRoot = resolve(projectRootInput || process.cwd());

  const candidateA = {
    repoRoot: projectRoot,
    scriptsDir: join(projectRoot, "skill manage", "scripts"),
    srcDir: join(projectRoot, "skill manage", "src")
  };

  const candidateB = {
    repoRoot: dirname(projectRoot),
    scriptsDir: join(projectRoot, "scripts"),
    srcDir: join(projectRoot, "src")
  };

  if (existsSync(candidateA.scriptsDir)) {
    return candidateA;
  }

  if (existsSync(candidateB.scriptsDir)) {
    return candidateB;
  }

  throw new Error(
    "Could not locate skill manage scripts directory. Run inside project root or pass --project <path>."
  );
}

function printRunSummary(label, result) {
  const outputPrefs = getOutputPreferences();
  const ok = result.code === 0;
  const icon = ok ? chalk.green("OK") : chalk.red("FAIL");
  const header = `${icon} ${chalk.bold(label)} | exit=${result.code}`;
  console.log(
    boxen(header, {
      padding: { top: 0, right: 1, bottom: 0, left: 1 },
      borderColor: ok ? "green" : "red",
      borderStyle: "round"
    })
  );

  if (result.code === 0) {
    console.log(chalk.green(`${label} completed successfully.`));
  } else {
    console.log(chalk.red(`${label} failed with exit code ${result.code}.`));
  }

  const stdoutToShow = toReadableOutput(label, result.stdout, outputPrefs);
  if (stdoutToShow.trim()) {
    console.log(chalk.gray("\nstdout:"));
    console.log(stdoutToShow.trim());
  }

  if (result.stderr.trim()) {
    console.log(chalk.yellow("\nstderr:"));
    console.log(result.stderr.trim());
  }
}

function printFeaturePanel() {
  const lines = [
    `${chalk.green("●")} changed-only src scanning`,
    `${chalk.green("●")} global-first sync strategy`,
    `${chalk.green("●")} one-command pipeline (aggregate + sync)`,
    `${chalk.green("●")} hardened input and command safety checks`
  ];
  console.log(
    boxen(lines.join("\n"), {
      padding: 1,
      borderStyle: "doubleSingle",
      borderColor: "cyan",
      title: "Feature Board",
      titleAlignment: "left"
    })
  );
}

function parseRepoNames(value) {
  return value
    .split(",")
    .map((x) => x.trim())
    .filter(Boolean);
}

function parseExcludeCategories(value) {
  if (value == null) return undefined;
  const trimmed = String(value).trim();
  if (!trimmed) return [];
  if (trimmed.toLowerCase() === "none") return [];

  return trimmed
    .split(",")
    .map((x) => x.trim().toLowerCase())
    .filter(Boolean);
}

function sanitizeExcludeCategories(values) {
  if (!values) return undefined;
  const normalized = [...new Set(values.map((x) => String(x).trim().toLowerCase()).filter(Boolean))];
  const invalid = normalized.filter((x) => !EXCLUSION_CATEGORY_CHOICES.includes(x));
  if (invalid.length > 0) {
    throw new Error(
      `Invalid exclude categories: ${invalid.join(", ")}. Allowed: ${EXCLUSION_CATEGORY_CHOICES.join(", ")}`
    );
  }
  return normalized;
}

async function executeAggregate(options) {
  const paths = resolveProjectPaths(options.project);
  const scriptPath = join(paths.scriptsDir, "aggregate-skills-to-subhubs.ps1");

  if (!existsSync(scriptPath)) {
    throw new Error(`Aggregate script not found at ${scriptPath}`);
  }

  const scriptArgs = [];
  if (options.dryRun) scriptArgs.push("-DryRun");
  if (options.allowMultiHub) scriptArgs.push("-AllowMultiHub");
  if (options.maxHubsPerSkill) scriptArgs.push("-MaxHubsPerSkill", String(options.maxHubsPerSkill));
  if (options.primaryMinScore) scriptArgs.push("-PrimaryMinScore", String(options.primaryMinScore));
  if (options.secondaryMinScore) scriptArgs.push("-SecondaryMinScore", String(options.secondaryMinScore));
  if (options.enableReviewBand) scriptArgs.push("-EnableReviewBand");
  if (options.reviewMinScore) scriptArgs.push("-ReviewMinScore", String(options.reviewMinScore));
  if (options.autoAcceptMinScore) scriptArgs.push("-AutoAcceptMinScore", String(options.autoAcceptMinScore));
  if (options.enableSemanticScoring) scriptArgs.push("-EnableSemanticScoring");
  if (options.semanticClassificationsFile) scriptArgs.push("-SemanticClassificationsFile", options.semanticClassificationsFile);
  if (options.semanticWeightFactor) scriptArgs.push("-SemanticWeightFactor", String(options.semanticWeightFactor));
  if (options.srcRepoMode) scriptArgs.push("-srcRepoMode", options.srcRepoMode);
  if (options.srcRepoNames?.length) {
    scriptArgs.push("-srcRepoNames", ...options.srcRepoNames);
  }
  if (options.noCategoryExclusions) {
    scriptArgs.push("-NoCategoryExclusions");
  }
  if (Array.isArray(options.excludeCategories) && options.excludeCategories.length > 0) {
    scriptArgs.push("-ExcludeCategories", options.excludeCategories.join(","));
  }

  const spinner = ora(chalk.cyan("Running aggregate workflow...")).start();
  const result = runPowerShellScript(scriptPath, scriptArgs, paths.repoRoot);
  spinner.stop();
  printRunSummary("Aggregate", result);

  if (result.code !== 0) {
    process.exit(result.code);
  }
}

async function executeSync(options) {
  const paths = resolveProjectPaths(options.project);
  const scriptPath = join(paths.scriptsDir, "sync-hubs.ps1");

  if (!existsSync(scriptPath)) {
    throw new Error(`Sync script not found at ${scriptPath}`);
  }

  const scriptArgs = ["-SyncMode", options.syncMode || "Auto"];
  if (options.force) scriptArgs.push("-Force");
  if (options.includeWorkspaceTargets) scriptArgs.push("-IncludeWorkspaceTargets");
  if (options.pruneWorkspaceTargets) scriptArgs.push("-PruneWorkspaceTargets");
  if (options.includeGlobal) scriptArgs.push("-IncludeGlobal");

  const spinner = ora(chalk.cyan("Running sync workflow...")).start();
  const result = runPowerShellScript(scriptPath, scriptArgs, paths.repoRoot);
  spinner.stop();
  printRunSummary("Sync", result);

  if (result.code !== 0) {
    process.exit(result.code);
  }
}

function executeAddsrc(repoUrl, options) {
  const paths = resolveProjectPaths(options.project);
  if (!isValidRepoUrl(repoUrl)) {
    throw new Error("Invalid repository URL. Use https://... or git@... format.");
  }

  if (!isSafeFolderName(options.name)) {
    throw new Error("Unsafe folder name. Use only letters, numbers, dot, underscore, or dash.");
  }

  if (!existsSync(paths.srcDir)) {
    mkdirSync(paths.srcDir, { recursive: true });
  }

  const spinner = ora(chalk.cyan("Cloning src repository...")).start();
  const args = ["clone", repoUrl];
  if (options.name) {
    const destinationPath = join(paths.srcDir, options.name);
    if (existsSync(destinationPath)) {
      spinner.fail("Destination already exists.");
      throw new Error(`Target folder already exists: ${destinationPath}`);
    }
    args.push(options.name);
  }

  const result = runCmd("git", args, paths.srcDir);
  spinner.stop();
  printRunSummary("Add src", result);

  if (result.code !== 0) {
    process.exit(result.code);
  }
}

function executeDoctor(options) {
  const paths = resolveProjectPaths(options.project);
  const checks = [
    ["Aggregate script", join(paths.scriptsDir, "aggregate-skills-to-subhubs.ps1")],
    ["Sync script", join(paths.scriptsDir, "sync-hubs.ps1")],
    ["src directory", paths.srcDir]
  ];

  console.log(chalk.bold("Doctor report"));
  for (const [name, path] of checks) {
    const ok = existsSync(path);
    const mark = ok ? chalk.green("OK") : chalk.red("MISSING");
    console.log(`${mark} ${name}: ${path}`);
  }

  const gitProbe = runCmd("git", ["--version"], paths.repoRoot, 30000);
  const gitMark = gitProbe.code === 0 ? chalk.green("OK") : chalk.red("MISSING");
  console.log(`${gitMark} Git executable`);

  const pwshProbe = runCmd("powershell", ["-NoProfile", "-Command", "$PSVersionTable.PSVersion.ToString()"], paths.repoRoot, 30000);
  const pwshMark = pwshProbe.code === 0 ? chalk.green("OK") : chalk.red("MISSING");
  console.log(`${pwshMark} PowerShell runtime`);
}

function executeSecurityAudit(options) {
  const paths = resolveProjectPaths(options.project);
  const findings = [];

  const cliPkgPath = join(paths.repoRoot, "skill manage", "cli", "package.json");
  if (existsSync(cliPkgPath)) {
    const pkgResult = runCmd("node", ["-e", `const fs=require('fs');const p='${cliPkgPath.replace(/\\/g, "\\\\")}';const x=JSON.parse(fs.readFileSync(p,'utf8'));const vals=[x.homepage,x?.repository?.url,x?.bugs?.url].filter(Boolean);const bad=vals.some(v=>String(v).includes('<your-username>'));process.stdout.write(bad?'bad':'ok');`], paths.repoRoot, 30000);
    if (pkgResult.stdout === "bad") {
      findings.push("package.json still contains placeholder GitHub URLs.");
    }
  }

  const probe = runPowerShellScript(join(paths.scriptsDir, "aggregate-skills-to-subhubs.ps1"), ["-DryRun", "-srcRepoMode", "changed-only"], paths.repoRoot);
  if (probe.code !== 0) {
    findings.push("aggregate dry-run failed in changed-only mode.");
  }

  if (findings.length === 0) {
    console.log(chalk.green("Security audit passed: no blocking findings."));
    return;
  }

  console.log(chalk.yellow("Security audit findings:"));
  for (const finding of findings) {
    console.log(chalk.yellow(`- ${finding}`));
  }
  process.exit(2);
}

async function executeInit(options) {
  const project = options.project;
  const srcRepoMode = options.srcRepoMode || "changed-only";
  const srcRepoNames = options.srcRepoNames || [];

  const doctorSpinner = ora(chalk.cyan("Checking project health...")).start();
  try {
    executeDoctor({ project });
    doctorSpinner.succeed("Project health check completed.");
  } catch (err) {
    doctorSpinner.fail("Project health check failed.");
    throw err;
  }

  if (options.repoUrl) {
    executeAddsrc(options.repoUrl, {
      project,
      name: options.name
    });
  }

  await executeAggregate({
    project,
    srcRepoMode,
    srcRepoNames,
    excludeCategories: options.excludeCategories,
    noCategoryExclusions: Boolean(options.noCategoryExclusions),
    dryRun: Boolean(options.dryRun)
  });

  if (!options.skipSync) {
    await executeSync({
      project,
      syncMode: options.syncMode || "Auto",
      force: true
    });
  }

  console.log(chalk.green("\nInitialization workflow finished."));
}

async function executeInteractive(options) {
  const project = options.project;

  printFeaturePanel();

  const action = await select({
    message: "Choose what to do",
    choices: [
      { name: "Initialize project (doctor + aggregate + sync)", value: "init" },
      { name: "Run full pipeline (aggregate + sync)", value: "run" },
      { name: "Aggregate only", value: "aggregate" },
      { name: "Sync only", value: "sync" },
      { name: "Add src repository", value: "add-src" },
      { name: "Doctor", value: "doctor" }
    ]
  });

  if (action === "doctor") {
    executeDoctor({ project });
    return;
  }

  if (action === "init") {
    const srcRepoMode = await select({
      message: "src repo mode",
      choices: [
        { name: "changed-only (recommended)", value: "changed-only" },
        { name: "latest", value: "latest" },
        { name: "selected", value: "selected" },
        { name: "all", value: "all" }
      ]
    });

    let srcRepoNames = [];
    if (srcRepoMode === "selected") {
      const rawNames = await input({
        message: "Comma-separated repository names",
        validate: (v) => (v.trim().length > 0 ? true : "Provide at least one repository name")
      });
      srcRepoNames = parseRepoNames(rawNames);
    }

    const skipSync = await confirm({
      message: "Skip sync step?",
      default: false
    });

    const selectedExclusions = await checkbox({
      message: "Exclude categories for this project run",
      choices: EXCLUSION_CATEGORY_CHOICES.map((x) => ({
        name: x,
        value: x,
        checked: true
      }))
    });

    const noCategoryExclusions = selectedExclusions.length === 0;

    await executeInit({
      project,
      srcRepoMode,
      srcRepoNames,
      excludeCategories: selectedExclusions,
      noCategoryExclusions,
      skipSync,
      syncMode: "Auto"
    });
    return;
  }

  if (action === "add-src") {
    const repoUrl = await input({
      message: "Repository URL",
      validate: (v) => (v?.startsWith("https://") || v?.startsWith("git@") ? true : "Enter a valid git URL")
    });
    const folderName = await input({
      message: "Custom folder name (optional)",
      default: ""
    });

    executeAddsrc(repoUrl, {
      project,
      name: folderName || undefined
    });
    return;
  }

  if (action === "sync") {
    const syncMode = await select({
      message: "Sync mode",
      choices: [
        { name: "Auto (recommended)", value: "Auto" },
        { name: "Junction", value: "Junction" },
        { name: "Copy", value: "Copy" },
        { name: "SymbolicLink", value: "SymbolicLink" }
      ]
    });

    const syncFlags = await checkbox({
      message: "Optional sync flags",
      choices: [
        { name: "Force overwrite", value: "force", checked: true },
        { name: "Include workspace targets", value: "includeWorkspaceTargets" },
        { name: "Prune workspace targets", value: "pruneWorkspaceTargets" }
      ]
    });

    await executeSync({
      project,
      syncMode,
      force: syncFlags.includes("force"),
      includeWorkspaceTargets: syncFlags.includes("includeWorkspaceTargets"),
      pruneWorkspaceTargets: syncFlags.includes("pruneWorkspaceTargets")
    });
    return;
  }

  const srcRepoMode = await select({
    message: "src repo mode",
    choices: [
      { name: "changed-only (recommended)", value: "changed-only" },
      { name: "latest", value: "latest" },
      { name: "selected", value: "selected" },
      { name: "all", value: "all" }
    ]
  });

  let srcRepoNames = [];
  if (srcRepoMode === "selected") {
    const rawNames = await input({
      message: "Comma-separated repository names",
      validate: (v) => (v.trim().length > 0 ? true : "Provide at least one repository name")
    });
    srcRepoNames = parseRepoNames(rawNames);
  }

  const allowMultiHub = await confirm({
    message: "Enable multi-hub classification?",
    default: false
  });

  const dryRun = await confirm({
    message: "Run in dry-run mode?",
    default: false
  });

  const selectedExclusions = await checkbox({
    message: "Exclude categories for this run",
    choices: EXCLUSION_CATEGORY_CHOICES.map((x) => ({
      name: x,
      value: x,
      checked: true
    }))
  });

  const noCategoryExclusions = selectedExclusions.length === 0;

  await executeAggregate({
    project,
    srcRepoMode,
    srcRepoNames,
    allowMultiHub,
    excludeCategories: selectedExclusions,
    noCategoryExclusions,
    dryRun
  });

  if (action === "run") {
    const syncMode = await select({
      message: "Sync mode",
      choices: [
        { name: "Auto (recommended)", value: "Auto" },
        { name: "Junction", value: "Junction" },
        { name: "Copy", value: "Copy" },
        { name: "SymbolicLink", value: "SymbolicLink" }
      ]
    });

    await executeSync({
      project,
      syncMode,
      force: true
    });
  }
}

printBanner();

program
  .name("skills-bank")
  .description("Visual CLI for AI Skills Bank")
  .version("0.1.0")
  .option("-p, --project <path>", "Project root path", process.cwd())
  .option("-v, --verbose", "Show verbose command output")
  .option("--raw-output", "Show raw script output without summarization");

program
  .command("init")
  .description("One-step setup: doctor, optional add-src, aggregate, and sync")
  .option("--src-repo-mode <mode>", "latest | all | selected | changed-only", "changed-only")
  .option("--src-repo-names <names>", "Comma-separated src repo names")
  .option("--repo-url <url>", "Optional src repository URL to clone before setup")
  .option("--name <folderName>", "Optional custom folder name when using --repo-url")
  .option("--dry-run", "Run aggregate in dry-run mode")
  .option("--exclude-categories <list>", `Comma-separated categories to exclude (${EXCLUSION_CATEGORY_CHOICES.join(" | ")})`)
  .option("--no-category-exclusions", "Disable all category exclusions")
  .option("--skip-sync", "Skip sync step")
  .option("--sync-mode <mode>", "Auto | Copy | Junction | SymbolicLink", "Auto")
  .action(async (cmdOpts) => {
    const global = program.opts();
    const srcRepoNames = cmdOpts.srcRepoNames ? parseRepoNames(cmdOpts.srcRepoNames) : [];
    const excludeCategories = sanitizeExcludeCategories(parseExcludeCategories(cmdOpts.excludeCategories));

    await executeInit({
      ...cmdOpts,
      project: global.project,
      srcRepoNames,
      excludeCategories,
      noCategoryExclusions: cmdOpts.categoryExclusions === false
    });
  });

program
  .command("aggregate")
  .description("Aggregate src skills into categorized hubs")
  .option("--dry-run", "Run without writing files")
  .option("--allow-multi-hub", "Enable multi-hub classification")
  .option("--max-hubs-per-skill <n>", "Max hubs per skill", Number)
  .option("--primary-min-score <n>", "Primary threshold score", Number)
  .option("--secondary-min-score <n>", "Secondary threshold score", Number)
  .option("--enable-review-band", "Enable review-band routing (mid-confidence skills go to review-candidates.ndjson)")
  .option("--review-min-score <n>", "Minimum score for review candidate", Number)
  .option("--auto-accept-min-score <n>", "Minimum score for automatic routing when review band is enabled", Number)
  .option("--enable-semantic-scoring", "Enable AI semantic understanding to improve skill routing")
  .option("--semantic-classifications-file <path>", "Path to semantic classifications JSON file")
  .option("--semantic-weight-factor <n>", "Weight factor for semantic scoring (0.0-1.0, default 0.6)", Number)
  .option("--src-repo-mode <mode>", "latest | all | selected | changed-only", "changed-only")
  .option("--src-repo-names <names>", "Comma-separated src repo names")
  .option("--exclude-categories <list>", `Comma-separated categories to exclude (${EXCLUSION_CATEGORY_CHOICES.join(" | ")})`)
  .option("--no-category-exclusions", "Disable all category exclusions")
  .action(async (cmdOpts) => {
    const global = program.opts();
    const srcRepoNames = cmdOpts.srcRepoNames ? parseRepoNames(cmdOpts.srcRepoNames) : [];
    const excludeCategories = sanitizeExcludeCategories(parseExcludeCategories(cmdOpts.excludeCategories));
    await executeAggregate({
      ...cmdOpts,
      project: global.project,
      srcRepoNames,
      excludeCategories,
      noCategoryExclusions: cmdOpts.categoryExclusions === false
    });
  });

program
  .command("sync")
  .description("Sync generated hubs into tool destinations")
  .option("--sync-mode <mode>", "Auto | Copy | Junction | SymbolicLink", "Auto")
  .option("--force", "Overwrite existing targets")
  .option("--include-workspace-targets", "Also include workspace-local targets")
  .option("--prune-workspace-targets", "Delete workspace-local targets")
  .option("--include-global", "Include global targets when custom target list is used")
  .action(async (cmdOpts) => {
    const global = program.opts();
    await executeSync({
      ...cmdOpts,
      project: global.project
    });
  });

program
  .command("add-src <repoUrl>")
  .description("Clone a src skill repository into src directory")
  .option("--name <folderName>", "Optional custom folder name")
  .action((repoUrl, cmdOpts) => {
    const global = program.opts();
    executeAddsrc(repoUrl, {
      ...cmdOpts,
      project: global.project
    });
  });

program
  .command("run")
  .description("Run aggregate then sync")
  .option("--enable-review-band", "Enable review-band routing (mid-confidence skills go to review-candidates.ndjson)")
  .option("--review-min-score <n>", "Minimum score for review candidate", Number)
  .option("--auto-accept-min-score <n>", "Minimum score for automatic routing when review band is enabled", Number)
  .option("--enable-semantic-scoring", "Enable AI semantic understanding to improve skill routing")
  .option("--semantic-classifications-file <path>", "Path to semantic classifications JSON file")
  .option("--semantic-weight-factor <n>", "Weight factor for semantic scoring (0.0-1.0, default 0.6)", Number)
  .option("--src-repo-mode <mode>", "latest | all | selected | changed-only", "changed-only")
  .option("--src-repo-names <names>", "Comma-separated src repo names")
  .option("--exclude-categories <list>", `Comma-separated categories to exclude (${EXCLUSION_CATEGORY_CHOICES.join(" | ")})`)
  .option("--no-category-exclusions", "Disable all category exclusions")
  .option("--sync-mode <mode>", "Auto | Copy | Junction | SymbolicLink", "Auto")
  .action(async (cmdOpts) => {
    const global = program.opts();
    const srcRepoNames = cmdOpts.srcRepoNames ? parseRepoNames(cmdOpts.srcRepoNames) : [];
    const excludeCategories = sanitizeExcludeCategories(parseExcludeCategories(cmdOpts.excludeCategories));

    await executeAggregate({
      project: global.project,
      enableReviewBand: Boolean(cmdOpts.enableReviewBand),
      reviewMinScore: cmdOpts.reviewMinScore,
      autoAcceptMinScore: cmdOpts.autoAcceptMinScore,
      enableSemanticScoring: Boolean(cmdOpts.enableSemanticScoring),
      semanticClassificationsFile: cmdOpts.semanticClassificationsFile,
      semanticWeightFactor: cmdOpts.semanticWeightFactor,
      srcRepoMode: cmdOpts.srcRepoMode,
      srcRepoNames,
      excludeCategories,
      noCategoryExclusions: cmdOpts.categoryExclusions === false
    });

    await executeSync({
      project: global.project,
      syncMode: cmdOpts.syncMode,
      force: true
    });
  });

program
  .command("doctor")
  .description("Check environment and required files")
  .action(() => {
    const global = program.opts();
    executeDoctor({ project: global.project });
  });

program
  .command("security")
  .description("Run security-focused checks and hardening validation")
  .action(() => {
    const global = program.opts();
    executeSecurityAudit({ project: global.project });
  });

program
  .command("interactive")
  .alias("i")
  .description("Interactive mode with guided prompts")
  .action(async () => {
    const global = program.opts();
    await executeInteractive({ project: global.project });
  });

program.parse(process.argv);
