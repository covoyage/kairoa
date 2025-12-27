<script lang="ts">
  import { translationsStore } from '$lib/stores/i18n';
  import { Copy, Check, Trash2, GitBranch, GitCommit, GitMerge, GitPullRequest, Download, Upload, Tag, History, Search } from 'lucide-svelte';

  let translations = $derived($translationsStore);

  function t(key: string): string {
    const keys = key.split('.');
    let value: any = translations;
    for (const k of keys) {
      value = value?.[k];
    }
    return value || key;
  }

  type CommandType = 'commit' | 'push' | 'pull' | 'fetch' | 'branch' | 'merge' | 'rebase' | 'tag' | 'log' | 'status' | 'clone' | 'remote' | 'stash' | 'reset' | 'checkout' | 'add' | 'diff' | 'show' | 'revert' | 'cherry-pick' | 'config' | 'init' | 'rm' | 'mv' | 'clean' | 'submodule' | 'worktree' | 'blame' | 'grep' | 'bisect' | 'reflog' | 'archive' | 'describe' | 'shortlog' | 'switch' | 'restore' | 'apply' | 'format-patch' | 'notes' | 'bundle' | 'sparse-checkout' | 'maintenance';

  let commandType = $state<CommandType>('commit');
  let generatedCommand = $state('');
  let commandDescription = $state('');
  let parameterDescriptions = $state<string[]>([]);
  let copied = $state(false);
  const STORAGE_KEY = 'gitCommands.state.v1';
  let hasLoadedFromStorage = false;

  // Commit 相关
  let commitMessage = $state('');
  let commitAll = $state(true);
  let commitAmend = $state(false);

  // Push 相关
  let pushRemote = $state('origin');
  let pushBranch = $state('');
  let pushForce = $state(false);
  let pushTags = $state(false);

  // Pull 相关
  let pullRemote = $state('origin');
  let pullBranch = $state('');
  let pullRebase = $state(false);

  // Branch 相关
  let branchName = $state('');
  let branchOperation = $state<'create' | 'delete' | 'rename' | 'list' | 'checkout'>('create');
  let branchOldName = $state('');
  let branchCheckout = $state(false);

  // Merge 相关
  let mergeBranch = $state('');
  let mergeNoFF = $state(false);
  let mergeSquash = $state(false);

  // Tag 相关
  let tagName = $state('');
  let tagMessage = $state('');
  let tagOperation = $state<'create' | 'delete' | 'list'>('create');
  let tagAnnotated = $state(true);

  // Log 相关
  let logCount = $state(10);
  let logGraph = $state(false);
  let logOneline = $state(false);
  let logAuthor = $state('');
  let logSearch = $state('');

  // Status 相关
  let statusShort = $state(false);

  // Clone 相关
  let cloneUrl = $state('');
  let cloneDirectory = $state('');
  let cloneDepth = $state('');
  let cloneBranch = $state('');

  // Remote 相关
  let remoteName = $state('origin');
  let remoteUrl = $state('');
  let remoteOperation = $state<'add' | 'remove' | 'list' | 'set-url'>('add');

  // Stash 相关
  let stashMessage = $state('');
  let stashOperation = $state<'save' | 'list' | 'pop' | 'apply' | 'drop'>('save');

  // Reset 相关
  let resetMode = $state<'soft' | 'mixed' | 'hard'>('mixed');
  let resetTarget = $state('HEAD');

  // Fetch 相关
  let fetchRemote = $state('origin');
  let fetchBranch = $state('');
  let fetchAll = $state(false);
  let fetchPrune = $state(false);
  let fetchTags = $state(false);

  // Checkout 相关
  let checkoutTarget = $state('');
  let checkoutBranch = $state('');
  let checkoutCreate = $state(false);
  let checkoutFile = $state('');

  // Add 相关
  let addFiles = $state('');
  let addAll = $state(false);
  let addPatch = $state(false);
  let addUpdate = $state(false);

  // Diff 相关
  let diffFile = $state('');
  let diffStaged = $state(false);
  let diffCached = $state(false);
  let diffCommit1 = $state('');
  let diffCommit2 = $state('');
  let diffStat = $state(false);

  // Show 相关
  let showCommit = $state('HEAD');
  let showFile = $state('');
  let showStat = $state(false);
  let showNameOnly = $state(false);

  // Revert 相关
  let revertCommit = $state('');
  let revertNoCommit = $state(false);
  let revertNoEdit = $state(false);

  // Cherry-pick 相关
  let cherryPickCommit = $state('');
  let cherryPickNoCommit = $state(false);
  let cherryPickEdit = $state(false);

  // Rebase 相关
  let rebaseBranch = $state('');
  let rebaseInteractive = $state(false);
  let rebaseOnto = $state('');
  let rebaseContinue = $state(false);
  let rebaseAbort = $state(false);

  // Config 相关
  let configKey = $state('');
  let configValue = $state('');
  let configOperation = $state<'get' | 'set' | 'unset' | 'list'>('get');
  let configGlobal = $state(false);
  let configLocal = $state(false);

  // Init 相关
  let initBare = $state(false);
  let initTemplate = $state('');
  let initDirectory = $state('');

  // Rm 相关
  let rmFiles = $state('');
  let rmCached = $state(false);
  let rmRecursive = $state(false);
  let rmForce = $state(false);

  // Mv 相关
  let mvSource = $state('');
  let mvDestination = $state('');

  // Clean 相关
  let cleanDryRun = $state(true);
  let cleanForce = $state(false);
  let cleanInteractive = $state(false);
  let cleanDirectory = $state(false);

  // Submodule 相关
  let submodulePath = $state('');
  let submoduleOperation = $state<'add' | 'update' | 'init' | 'deinit' | 'status'>('add');
  let submoduleUrl = $state('');
  let submoduleRecursive = $state(false);

  // Worktree 相关
  let worktreePath = $state('');
  let worktreeBranch = $state('');
  let worktreeOperation = $state<'add' | 'list' | 'remove' | 'prune'>('add');

  // Blame 相关
  let blameFile = $state('');
  let blameLineStart = $state('');
  let blameLineEnd = $state('');
  let blameShowEmail = $state(false);
  let blameShowLineNumbers = $state(true);

  // Grep 相关
  let grepPattern = $state('');
  let grepFile = $state('');
  let grepCaseInsensitive = $state(false);
  let grepRecursive = $state(true);
  let grepShowLineNumbers = $state(true);
  let grepExtendedRegex = $state(false);

  // Bisect 相关
  let bisectOperation = $state<'start' | 'good' | 'bad' | 'skip' | 'reset' | 'run'>('start');
  let bisectCommit = $state('');
  let bisectScript = $state('');

  // Reflog 相关
  let reflogShowAll = $state(false);
  let reflogCount = $state(10);
  let reflogRef = $state('');

  // Archive 相关
  let archiveFormat = $state<'tar' | 'zip' | 'tar.gz'>('tar.gz');
  let archiveOutput = $state('');
  let archiveTree = $state('HEAD');
  let archivePrefix = $state('');

  // Describe 相关
  let describeCommit = $state('HEAD');
  let describeTags = $state(false);
  let describeAll = $state(false);
  let describeLong = $state(false);

  // Shortlog 相关
  let shortlogCount = $state(10);
  let shortlogEmail = $state(false);
  let shortlogGroup = $state<'author' | 'committer'>('author');

  // Switch 相关
  let switchBranch = $state('');
  let switchCreate = $state(false);
  let switchTrack = $state(false);
  let switchDetach = $state(false);

  // Restore 相关
  let restoreSource = $state('');
  let restoreStaged = $state(false);
  let restoreWorktree = $state(false);
  let restoreFile = $state('');

  // Apply 相关
  let applyPatch = $state('');
  let applyCheck = $state(false);
  let applyReverse = $state(false);
  let apply3way = $state(false);

  // Format-patch 相关
  let formatPatchRange = $state('');
  let formatPatchOutput = $state('');
  let formatPatchNumbered = $state(true);
  let formatPatchCoverLetter = $state(false);

  // Notes 相关
  let notesOperation = $state<'add' | 'show' | 'list' | 'remove' | 'append'>('add');
  let notesCommit = $state('HEAD');
  let notesMessage = $state('');
  let notesRef = $state('refs/notes/commits');

  // Bundle 相关
  let bundleOperation = $state<'create' | 'list-heads' | 'verify' | 'unbundle'>('create');
  let bundleFile = $state('');
  let bundleBranch = $state('');
  let bundleAll = $state(false);

  // Sparse-checkout 相关
  let sparseCheckoutOperation = $state<'init' | 'set' | 'add' | 'disable' | 'list'>('init');
  let sparseCheckoutPaths = $state('');
  let sparseCheckoutCone = $state(true);

  // Maintenance 相关
  let maintenanceOperation = $state<'start' | 'stop' | 'run'>('run');
  let maintenanceTask = $state<'gc' | 'commit-graph' | 'prefetch' | 'loose-objects' | 'incremental-repack'>('gc');

  // 从本地存储恢复状态
  function loadSavedState() {
    if (hasLoadedFromStorage) return;
    hasLoadedFromStorage = true;
    try {
      const saved = localStorage.getItem(STORAGE_KEY);
      if (saved) {
        const parsed = JSON.parse(saved);
        commandType = parsed.commandType || 'commit';
        commitMessage = parsed.commitMessage || '';
        pushRemote = parsed.pushRemote || 'origin';
        pullRemote = parsed.pullRemote || 'origin';
        branchName = parsed.branchName || '';
        mergeBranch = parsed.mergeBranch || '';
        tagName = parsed.tagName || '';
        cloneUrl = parsed.cloneUrl || '';
        remoteName = parsed.remoteName || 'origin';
        remoteUrl = parsed.remoteUrl || '';
      }
    } catch (error) {
      console.error('Failed to load Git commands state:', error);
    }
  }

  // 保存状态
  function saveState() {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify({
        commandType,
        commitMessage,
        pushRemote,
        pullRemote,
        branchName,
        mergeBranch,
        tagName,
        cloneUrl,
        remoteName,
        remoteUrl
      }));
    } catch (error) {
      console.error('Failed to save Git commands state:', error);
    }
  }

  // 初始化时加载保存的状态
  loadSavedState();

  // 命令类型变化时保存和更新说明
  $effect(() => {
    commandType;
    commandDescription = getCommandDescription(commandType);
    parameterDescriptions = getParameterDescriptions(commandType);
    saveState();
  });

  function generateCommand() {
    let cmd = '';

    switch (commandType) {
      case 'commit':
        cmd = 'git commit';
        if (commitAmend) {
          cmd += ' --amend';
        }
        if (commitMessage) {
          cmd += ` -m "${commitMessage}"`;
        } else if (!commitAmend) {
          cmd += ' -m "Your commit message"';
        }
        if (commitAll && !commitAmend) {
          cmd = 'git add -A && ' + cmd;
        }
        break;

      case 'push':
        cmd = 'git push';
        if (pushForce) {
          cmd += ' --force';
        }
        if (pushTags) {
          cmd += ' --tags';
        }
        if (pushRemote) {
          cmd += ` ${pushRemote}`;
        }
        if (pushBranch) {
          cmd += ` ${pushBranch}`;
        }
        break;

      case 'pull':
        cmd = 'git pull';
        if (pullRebase) {
          cmd += ' --rebase';
        }
        if (pullRemote) {
          cmd += ` ${pullRemote}`;
        }
        if (pullBranch) {
          cmd += ` ${pullBranch}`;
        }
        break;

      case 'branch':
        if (branchOperation === 'create') {
          if (branchCheckout) {
            cmd = `git checkout -b ${branchName || '<branch-name>'}`;
          } else {
            cmd = `git branch ${branchName || '<branch-name>'}`;
          }
        } else if (branchOperation === 'delete') {
          cmd = `git branch -d ${branchName || '<branch-name>'}`;
        } else if (branchOperation === 'rename') {
          cmd = `git branch -m ${branchOldName || '<old-name>'} ${branchName || '<new-name>'}`;
        } else if (branchOperation === 'list') {
          cmd = 'git branch';
          if (branchCheckout) {
            cmd += ' -a';
          }
        } else if (branchOperation === 'checkout') {
          cmd = `git checkout ${branchName || '<branch-name>'}`;
        }
        break;

      case 'merge':
        cmd = 'git merge';
        if (mergeNoFF) {
          cmd += ' --no-ff';
        }
        if (mergeSquash) {
          cmd += ' --squash';
        }
        cmd += ` ${mergeBranch || '<branch-name>'}`;
        break;

      case 'tag':
        if (tagOperation === 'create') {
          cmd = 'git tag';
          if (tagAnnotated) {
            cmd += ' -a';
          }
          cmd += ` ${tagName || '<tag-name>'}`;
          if (tagMessage && tagAnnotated) {
            cmd += ` -m "${tagMessage}"`;
          }
        } else if (tagOperation === 'delete') {
          cmd = `git tag -d ${tagName || '<tag-name>'}`;
        } else if (tagOperation === 'list') {
          cmd = 'git tag';
        }
        break;

      case 'log':
        cmd = 'git log';
        if (logCount) {
          cmd += ` -${logCount}`;
        }
        if (logGraph) {
          cmd += ' --graph';
        }
        if (logOneline) {
          cmd += ' --oneline';
        }
        if (logAuthor) {
          cmd += ` --author="${logAuthor}"`;
        }
        if (logSearch) {
          cmd += ` --grep="${logSearch}"`;
        }
        break;

      case 'status':
        cmd = 'git status';
        if (statusShort) {
          cmd += ' -s';
        }
        break;

      case 'clone':
        cmd = 'git clone';
        if (cloneDepth) {
          cmd += ` --depth ${cloneDepth}`;
        }
        if (cloneBranch) {
          cmd += ` -b ${cloneBranch}`;
        }
        cmd += ` ${cloneUrl || '<repository-url>'}`;
        if (cloneDirectory) {
          cmd += ` ${cloneDirectory}`;
        }
        break;

      case 'remote':
        if (remoteOperation === 'add') {
          cmd = `git remote add ${remoteName || 'origin'} ${remoteUrl || '<repository-url>'}`;
        } else if (remoteOperation === 'remove') {
          cmd = `git remote remove ${remoteName || 'origin'}`;
        } else if (remoteOperation === 'list') {
          cmd = 'git remote -v';
        } else if (remoteOperation === 'set-url') {
          cmd = `git remote set-url ${remoteName || 'origin'} ${remoteUrl || '<repository-url>'}`;
        }
        break;

      case 'stash':
        if (stashOperation === 'save') {
          cmd = 'git stash';
          if (stashMessage) {
            cmd += ` save "${stashMessage}"`;
          }
        } else if (stashOperation === 'list') {
          cmd = 'git stash list';
        } else if (stashOperation === 'pop') {
          cmd = 'git stash pop';
        } else if (stashOperation === 'apply') {
          cmd = 'git stash apply';
        } else if (stashOperation === 'drop') {
          cmd = 'git stash drop';
        }
        break;

      case 'reset':
        cmd = `git reset --${resetMode} ${resetTarget || 'HEAD'}`;
        break;

      case 'fetch':
        cmd = 'git fetch';
        if (fetchAll) {
          cmd += ' --all';
        }
        if (fetchPrune) {
          cmd += ' --prune';
        }
        if (fetchTags) {
          cmd += ' --tags';
        }
        if (fetchRemote && !fetchAll) {
          cmd += ` ${fetchRemote}`;
        }
        if (fetchBranch && !fetchAll) {
          cmd += ` ${fetchBranch}`;
        }
        break;

      case 'checkout':
        if (checkoutFile) {
          cmd = `git checkout ${checkoutFile}`;
        } else if (checkoutCreate) {
          cmd = `git checkout -b ${checkoutBranch || checkoutTarget || '<branch-name>'}`;
        } else {
          cmd = `git checkout ${checkoutTarget || checkoutBranch || '<branch-or-file>'}`;
        }
        break;

      case 'add':
        cmd = 'git add';
        if (addAll) {
          cmd += ' -A';
        } else if (addPatch) {
          cmd += ' -p';
        } else if (addUpdate) {
          cmd += ' -u';
        }
        if (addFiles && !addAll && !addPatch && !addUpdate) {
          cmd += ` ${addFiles}`;
        } else if (!addAll && !addPatch && !addUpdate) {
          cmd += ' <file>...';
        }
        break;

      case 'diff':
        cmd = 'git diff';
        if (diffStaged || diffCached) {
          cmd += ' --staged';
        }
        if (diffStat) {
          cmd += ' --stat';
        }
        if (diffCommit1) {
          cmd += ` ${diffCommit1}`;
        }
        if (diffCommit2) {
          cmd += ` ${diffCommit2}`;
        }
        if (diffFile && !diffCommit1 && !diffCommit2) {
          cmd += ` ${diffFile}`;
        }
        break;

      case 'show':
        cmd = 'git show';
        if (showStat) {
          cmd += ' --stat';
        }
        if (showNameOnly) {
          cmd += ' --name-only';
        }
        cmd += ` ${showCommit || 'HEAD'}`;
        if (showFile) {
          cmd += `:${showFile}`;
        }
        break;

      case 'revert':
        cmd = 'git revert';
        if (revertNoCommit) {
          cmd += ' --no-commit';
        }
        if (revertNoEdit) {
          cmd += ' --no-edit';
        }
        cmd += ` ${revertCommit || '<commit>'}`;
        break;

      case 'cherry-pick':
        cmd = 'git cherry-pick';
        if (cherryPickNoCommit) {
          cmd += ' --no-commit';
        }
        if (cherryPickEdit) {
          cmd += ' --edit';
        }
        cmd += ` ${cherryPickCommit || '<commit>'}`;
        break;

      case 'rebase':
        if (rebaseContinue) {
          cmd = 'git rebase --continue';
        } else if (rebaseAbort) {
          cmd = 'git rebase --abort';
        } else {
          cmd = 'git rebase';
          if (rebaseInteractive) {
            cmd += ' -i';
          }
          if (rebaseOnto) {
            cmd += ` --onto ${rebaseOnto}`;
          }
          cmd += ` ${rebaseBranch || '<branch>'}`;
        }
        break;

      case 'config':
        if (configOperation === 'get') {
          cmd = 'git config';
          if (configGlobal) {
            cmd += ' --global';
          } else if (configLocal) {
            cmd += ' --local';
          }
          cmd += ` ${configKey || '<key>'}`;
        } else if (configOperation === 'set') {
          cmd = 'git config';
          if (configGlobal) {
            cmd += ' --global';
          } else if (configLocal) {
            cmd += ' --local';
          }
          cmd += ` ${configKey || '<key>'} "${configValue || '<value>'}"`;
        } else if (configOperation === 'unset') {
          cmd = 'git config';
          if (configGlobal) {
            cmd += ' --global';
          } else if (configLocal) {
            cmd += ' --local';
          }
          cmd += ` --unset ${configKey || '<key>'}`;
        } else if (configOperation === 'list') {
          cmd = 'git config --list';
          if (configGlobal) {
            cmd += ' --global';
          } else if (configLocal) {
            cmd += ' --local';
          }
        }
        break;

      case 'init':
        cmd = 'git init';
        if (initBare) {
          cmd += ' --bare';
        }
        if (initTemplate) {
          cmd += ` --template=${initTemplate}`;
        }
        if (initDirectory) {
          cmd += ` ${initDirectory}`;
        }
        break;

      case 'rm':
        cmd = 'git rm';
        if (rmCached) {
          cmd += ' --cached';
        }
        if (rmRecursive) {
          cmd += ' -r';
        }
        if (rmForce) {
          cmd += ' -f';
        }
        cmd += ` ${rmFiles || '<file>...'}`;
        break;

      case 'mv':
        cmd = `git mv ${mvSource || '<source>'} ${mvDestination || '<destination>'}`;
        break;

      case 'clean':
        cmd = 'git clean';
        if (cleanDryRun) {
          cmd += ' -n';
        }
        if (cleanForce) {
          cmd += ' -f';
        }
        if (cleanInteractive) {
          cmd += ' -i';
        }
        if (cleanDirectory) {
          cmd += ' -d';
        }
        break;

      case 'submodule':
        if (submoduleOperation === 'add') {
          cmd = 'git submodule add';
          if (submoduleUrl) {
            cmd += ` ${submoduleUrl}`;
          }
          if (submodulePath) {
            cmd += ` ${submodulePath}`;
          }
        } else if (submoduleOperation === 'update') {
          cmd = 'git submodule update';
          if (submoduleRecursive) {
            cmd += ' --recursive';
          }
          if (submodulePath) {
            cmd += ` ${submodulePath}`;
          }
        } else if (submoduleOperation === 'init') {
          cmd = 'git submodule init';
          if (submodulePath) {
            cmd += ` ${submodulePath}`;
          }
        } else if (submoduleOperation === 'deinit') {
          cmd = 'git submodule deinit';
          if (submodulePath) {
            cmd += ` ${submodulePath}`;
          }
        } else if (submoduleOperation === 'status') {
          cmd = 'git submodule status';
        }
        break;

      case 'worktree':
        if (worktreeOperation === 'add') {
          cmd = 'git worktree add';
          if (worktreePath) {
            cmd += ` ${worktreePath}`;
          }
          if (worktreeBranch) {
            cmd += ` -b ${worktreeBranch}`;
          }
        } else if (worktreeOperation === 'list') {
          cmd = 'git worktree list';
        } else if (worktreeOperation === 'remove') {
          cmd = `git worktree remove ${worktreePath || '<path>'}`;
        } else if (worktreeOperation === 'prune') {
          cmd = 'git worktree prune';
        }
        break;

      case 'blame':
        cmd = 'git blame';
        if (blameShowEmail) {
          cmd += ' -e';
        }
        if (!blameShowLineNumbers) {
          cmd += ' -n';
        }
        if (blameLineStart && blameLineEnd) {
          cmd += ` -L ${blameLineStart},${blameLineEnd}`;
        } else if (blameLineStart) {
          cmd += ` -L ${blameLineStart}`;
        }
        cmd += ` ${blameFile || '<file>'}`;
        break;

      case 'grep':
        cmd = 'git grep';
        if (grepCaseInsensitive) {
          cmd += ' -i';
        }
        if (grepRecursive) {
          cmd += ' -r';
        }
        if (!grepShowLineNumbers) {
          cmd += ' -n';
        }
        if (grepExtendedRegex) {
          cmd += ' -E';
        }
        cmd += ` "${grepPattern || '<pattern>'}"`;
        if (grepFile) {
          cmd += ` ${grepFile}`;
        }
        break;

      case 'bisect':
        if (bisectOperation === 'start') {
          cmd = 'git bisect start';
          if (bisectCommit) {
            cmd += ` ${bisectCommit}`;
          }
        } else if (bisectOperation === 'good') {
          cmd = `git bisect good ${bisectCommit || '<commit>'}`;
        } else if (bisectOperation === 'bad') {
          cmd = `git bisect bad ${bisectCommit || '<commit>'}`;
        } else if (bisectOperation === 'skip') {
          cmd = `git bisect skip ${bisectCommit || '<commit>'}`;
        } else if (bisectOperation === 'reset') {
          cmd = 'git bisect reset';
        } else if (bisectOperation === 'run') {
          cmd = `git bisect run ${bisectScript || '<script>'}`;
        }
        break;

      case 'reflog':
        cmd = 'git reflog';
        if (reflogShowAll) {
          cmd += ' --all';
        }
        if (reflogCount) {
          cmd += ` -n ${reflogCount}`;
        }
        if (reflogRef) {
          cmd += ` ${reflogRef}`;
        }
        break;

      case 'archive':
        cmd = 'git archive';
        if (archiveFormat === 'zip') {
          cmd += ' --format=zip';
        } else if (archiveFormat === 'tar') {
          cmd += ' --format=tar';
        } else {
          cmd += ' --format=tar.gz';
        }
        if (archivePrefix) {
          cmd += ` --prefix=${archivePrefix}/`;
        }
        cmd += ` -o ${archiveOutput || '<output-file>'}`;
        cmd += ` ${archiveTree || 'HEAD'}`;
        break;

      case 'describe':
        cmd = 'git describe';
        if (describeTags) {
          cmd += ' --tags';
        }
        if (describeAll) {
          cmd += ' --all';
        }
        if (describeLong) {
          cmd += ' --long';
        }
        cmd += ` ${describeCommit || 'HEAD'}`;
        break;

      case 'shortlog':
        cmd = 'git shortlog';
        if (shortlogCount) {
          cmd += ` -n ${shortlogCount}`;
        }
        if (shortlogEmail) {
          cmd += ' -e';
        }
        if (shortlogGroup === 'committer') {
          cmd += ' -c';
        }
        break;

      case 'switch':
        cmd = 'git switch';
        if (switchCreate) {
          cmd += ' -c';
        }
        if (switchTrack) {
          cmd += ' --track';
        }
        if (switchDetach) {
          cmd += ' --detach';
        }
        cmd += ` ${switchBranch || '<branch>'}`;
        break;

      case 'restore':
        cmd = 'git restore';
        if (restoreStaged) {
          cmd += ' --staged';
        }
        if (restoreWorktree) {
          cmd += ' --worktree';
        }
        if (restoreSource) {
          cmd += ` --source=${restoreSource}`;
        }
        cmd += ` ${restoreFile || '<file>...'}`;
        break;

      case 'apply':
        cmd = 'git apply';
        if (applyCheck) {
          cmd += ' --check';
        }
        if (applyReverse) {
          cmd += ' --reverse';
        }
        if (apply3way) {
          cmd += ' --3way';
        }
        cmd += ` ${applyPatch || '<patch-file>'}`;
        break;

      case 'format-patch':
        cmd = 'git format-patch';
        if (formatPatchNumbered) {
          cmd += ' --numbered';
        }
        if (formatPatchCoverLetter) {
          cmd += ' --cover-letter';
        }
        if (formatPatchOutput) {
          cmd += ` -o ${formatPatchOutput}`;
        }
        cmd += ` ${formatPatchRange || '<commit-range>'}`;
        break;

      case 'notes':
        if (notesOperation === 'add') {
          cmd = 'git notes add';
          if (notesMessage) {
            cmd += ` -m "${notesMessage}"`;
          }
          cmd += ` ${notesCommit || 'HEAD'}`;
        } else if (notesOperation === 'show') {
          cmd = `git notes show ${notesCommit || 'HEAD'}`;
        } else if (notesOperation === 'list') {
          cmd = 'git notes list';
          if (notesCommit && notesCommit !== 'HEAD') {
            cmd += ` ${notesCommit}`;
          }
        } else if (notesOperation === 'remove') {
          cmd = `git notes remove ${notesCommit || 'HEAD'}`;
        } else if (notesOperation === 'append') {
          cmd = 'git notes append';
          if (notesMessage) {
            cmd += ` -m "${notesMessage}"`;
          }
          cmd += ` ${notesCommit || 'HEAD'}`;
        }
        if (notesRef && notesRef !== 'refs/notes/commits') {
          cmd += ` --ref=${notesRef}`;
        }
        break;

      case 'bundle':
        if (bundleOperation === 'create') {
          cmd = `git bundle create ${bundleFile || '<file>'} ${bundleBranch || '<branch>'}`;
          if (bundleAll) {
            cmd = `git bundle create ${bundleFile || '<file>'} --all`;
          }
        } else if (bundleOperation === 'list-heads') {
          cmd = `git bundle list-heads ${bundleFile || '<file>'}`;
        } else if (bundleOperation === 'verify') {
          cmd = `git bundle verify ${bundleFile || '<file>'}`;
        } else if (bundleOperation === 'unbundle') {
          cmd = `git bundle unbundle ${bundleFile || '<file>'}`;
        }
        break;

      case 'sparse-checkout':
        if (sparseCheckoutOperation === 'init') {
          cmd = 'git sparse-checkout init';
          if (sparseCheckoutCone) {
            cmd += ' --cone';
          }
        } else if (sparseCheckoutOperation === 'set') {
          cmd = `git sparse-checkout set ${sparseCheckoutPaths || '<path>...'}`;
        } else if (sparseCheckoutOperation === 'add') {
          cmd = `git sparse-checkout add ${sparseCheckoutPaths || '<path>...'}`;
        } else if (sparseCheckoutOperation === 'disable') {
          cmd = 'git sparse-checkout disable';
        } else if (sparseCheckoutOperation === 'list') {
          cmd = 'git sparse-checkout list';
        }
        break;

      case 'maintenance':
        if (maintenanceOperation === 'start') {
          cmd = 'git maintenance start';
        } else if (maintenanceOperation === 'stop') {
          cmd = 'git maintenance stop';
        } else if (maintenanceOperation === 'run') {
          cmd = `git maintenance run --task=${maintenanceTask}`;
        }
        break;
    }

    generatedCommand = cmd;
    commandDescription = getCommandDescription(commandType);
    parameterDescriptions = getParameterDescriptions(commandType);
  }

  function getCommandDescription(type: CommandType): string {
    return t(`gitCommands.descriptions.${type}`);
  }

  function getParameterDescriptions(type: CommandType): string[] {
    const descriptions: string[] = [];
    
    switch (type) {
      case 'commit':
        if (commitAll) descriptions.push(t('gitCommands.paramDescriptions.commit.addAll'));
        if (commitAmend) descriptions.push(t('gitCommands.paramDescriptions.commit.amend'));
        if (commitMessage) descriptions.push(t('gitCommands.paramDescriptions.commit.message'));
        break;
      case 'push':
        if (pushForce) descriptions.push(t('gitCommands.paramDescriptions.push.force'));
        if (pushTags) descriptions.push(t('gitCommands.paramDescriptions.push.tags'));
        if (pushRemote) descriptions.push(t('gitCommands.paramDescriptions.push.remote'));
        if (pushBranch) descriptions.push(t('gitCommands.paramDescriptions.push.branch'));
        break;
      case 'pull':
        if (pullRebase) descriptions.push(t('gitCommands.paramDescriptions.pull.rebase'));
        if (pullRemote) descriptions.push(t('gitCommands.paramDescriptions.pull.remote'));
        if (pullBranch) descriptions.push(t('gitCommands.paramDescriptions.pull.branch'));
        break;
      case 'fetch':
        if (fetchAll) descriptions.push(t('gitCommands.paramDescriptions.fetch.all'));
        if (fetchPrune) descriptions.push(t('gitCommands.paramDescriptions.fetch.prune'));
        if (fetchTags) descriptions.push(t('gitCommands.paramDescriptions.fetch.tags'));
        if (fetchRemote) descriptions.push(t('gitCommands.paramDescriptions.fetch.remote'));
        if (fetchBranch) descriptions.push(t('gitCommands.paramDescriptions.fetch.branch'));
        break;
      case 'branch':
        if (branchCheckout && branchOperation === 'create') descriptions.push(t('gitCommands.paramDescriptions.branch.checkout'));
        if (branchCheckout && branchOperation === 'list') descriptions.push(t('gitCommands.paramDescriptions.branch.showAll'));
        break;
      case 'merge':
        if (mergeNoFF) descriptions.push(t('gitCommands.paramDescriptions.merge.noFF'));
        if (mergeSquash) descriptions.push(t('gitCommands.paramDescriptions.merge.squash'));
        if (mergeBranch) descriptions.push(t('gitCommands.paramDescriptions.merge.branch'));
        break;
      case 'rebase':
        if (rebaseInteractive) descriptions.push(t('gitCommands.paramDescriptions.rebase.interactive'));
        if (rebaseOnto) descriptions.push(t('gitCommands.paramDescriptions.rebase.onto'));
        if (rebaseContinue) descriptions.push(t('gitCommands.paramDescriptions.rebase.continue'));
        if (rebaseAbort) descriptions.push(t('gitCommands.paramDescriptions.rebase.abort'));
        if (rebaseBranch) descriptions.push(t('gitCommands.paramDescriptions.rebase.branch'));
        break;
      case 'tag':
        if (tagAnnotated) descriptions.push(t('gitCommands.paramDescriptions.tag.annotated'));
        if (tagMessage) descriptions.push(t('gitCommands.paramDescriptions.tag.message'));
        break;
      case 'log':
        if (logCount) descriptions.push(t('gitCommands.paramDescriptions.log.count'));
        if (logGraph) descriptions.push(t('gitCommands.paramDescriptions.log.graph'));
        if (logOneline) descriptions.push(t('gitCommands.paramDescriptions.log.oneline'));
        if (logAuthor) descriptions.push(t('gitCommands.paramDescriptions.log.author'));
        if (logSearch) descriptions.push(t('gitCommands.paramDescriptions.log.search'));
        break;
      case 'status':
        if (statusShort) descriptions.push(t('gitCommands.paramDescriptions.status.short'));
        break;
      case 'clone':
        if (cloneDepth) descriptions.push(t('gitCommands.paramDescriptions.clone.depth'));
        if (cloneBranch) descriptions.push(t('gitCommands.paramDescriptions.clone.branch'));
        if (cloneDirectory) descriptions.push(t('gitCommands.paramDescriptions.clone.directory'));
        break;
      case 'checkout':
        if (checkoutCreate) descriptions.push(t('gitCommands.paramDescriptions.checkout.create'));
        if (checkoutFile) descriptions.push(t('gitCommands.paramDescriptions.checkout.file'));
        break;
      case 'add':
        if (addAll) descriptions.push(t('gitCommands.paramDescriptions.add.all'));
        if (addPatch) descriptions.push(t('gitCommands.paramDescriptions.add.patch'));
        if (addUpdate) descriptions.push(t('gitCommands.paramDescriptions.add.update'));
        if (addFiles) descriptions.push(t('gitCommands.paramDescriptions.add.files'));
        break;
      case 'diff':
        if (diffStaged || diffCached) descriptions.push(t('gitCommands.paramDescriptions.diff.staged'));
        if (diffStat) descriptions.push(t('gitCommands.paramDescriptions.diff.stat'));
        if (diffCommit1) descriptions.push(t('gitCommands.paramDescriptions.diff.commit1'));
        if (diffCommit2) descriptions.push(t('gitCommands.paramDescriptions.diff.commit2'));
        if (diffFile) descriptions.push(t('gitCommands.paramDescriptions.diff.file'));
        break;
      case 'show':
        if (showStat) descriptions.push(t('gitCommands.paramDescriptions.show.stat'));
        if (showNameOnly) descriptions.push(t('gitCommands.paramDescriptions.show.nameOnly'));
        if (showFile) descriptions.push(t('gitCommands.paramDescriptions.show.file'));
        break;
      case 'revert':
        if (revertNoCommit) descriptions.push(t('gitCommands.paramDescriptions.revert.noCommit'));
        if (revertNoEdit) descriptions.push(t('gitCommands.paramDescriptions.revert.noEdit'));
        break;
      case 'cherry-pick':
        if (cherryPickNoCommit) descriptions.push(t('gitCommands.paramDescriptions.cherryPick.noCommit'));
        if (cherryPickEdit) descriptions.push(t('gitCommands.paramDescriptions.cherryPick.edit'));
        break;
      case 'reset':
        descriptions.push(t(`gitCommands.paramDescriptions.reset.${resetMode}`));
        if (resetTarget && resetTarget !== 'HEAD') descriptions.push(t('gitCommands.paramDescriptions.reset.target'));
        break;
      case 'stash':
        if (stashMessage) descriptions.push(t('gitCommands.paramDescriptions.stash.message'));
        break;
      case 'config':
        if (configGlobal) descriptions.push(t('gitCommands.paramDescriptions.config.global'));
        if (configLocal) descriptions.push(t('gitCommands.paramDescriptions.config.local'));
        if (configKey) descriptions.push(t('gitCommands.paramDescriptions.config.key'));
        if (configValue) descriptions.push(t('gitCommands.paramDescriptions.config.value'));
        break;
      case 'init':
        if (initBare) descriptions.push(t('gitCommands.paramDescriptions.init.bare'));
        if (initTemplate) descriptions.push(t('gitCommands.paramDescriptions.init.template'));
        break;
      case 'rm':
        if (rmCached) descriptions.push(t('gitCommands.paramDescriptions.rm.cached'));
        if (rmRecursive) descriptions.push(t('gitCommands.paramDescriptions.rm.recursive'));
        if (rmForce) descriptions.push(t('gitCommands.paramDescriptions.rm.force'));
        break;
      case 'clean':
        if (cleanDryRun) descriptions.push(t('gitCommands.paramDescriptions.clean.dryRun'));
        if (cleanForce) descriptions.push(t('gitCommands.paramDescriptions.clean.force'));
        if (cleanInteractive) descriptions.push(t('gitCommands.paramDescriptions.clean.interactive'));
        if (cleanDirectory) descriptions.push(t('gitCommands.paramDescriptions.clean.directory'));
        break;
      case 'blame':
        if (blameShowEmail) descriptions.push(t('gitCommands.paramDescriptions.blame.showEmail'));
        if (!blameShowLineNumbers) descriptions.push(t('gitCommands.paramDescriptions.blame.noLineNumbers'));
        if (blameLineStart || blameLineEnd) descriptions.push(t('gitCommands.paramDescriptions.blame.lineRange'));
        break;
      case 'grep':
        if (grepCaseInsensitive) descriptions.push(t('gitCommands.paramDescriptions.grep.caseInsensitive'));
        if (grepRecursive) descriptions.push(t('gitCommands.paramDescriptions.grep.recursive'));
        if (!grepShowLineNumbers) descriptions.push(t('gitCommands.paramDescriptions.grep.noLineNumbers'));
        if (grepExtendedRegex) descriptions.push(t('gitCommands.paramDescriptions.grep.extendedRegex'));
        break;
      case 'reflog':
        if (reflogShowAll) descriptions.push(t('gitCommands.paramDescriptions.reflog.showAll'));
        if (reflogCount && reflogCount !== 10) descriptions.push(t('gitCommands.paramDescriptions.reflog.count'));
        break;
      case 'archive':
        if (archiveFormat !== 'tar.gz') descriptions.push(t(`gitCommands.paramDescriptions.archive.${archiveFormat}`));
        if (archivePrefix) descriptions.push(t('gitCommands.paramDescriptions.archive.prefix'));
        break;
      case 'describe':
        if (describeTags) descriptions.push(t('gitCommands.paramDescriptions.describe.tags'));
        if (describeAll) descriptions.push(t('gitCommands.paramDescriptions.describe.all'));
        if (describeLong) descriptions.push(t('gitCommands.paramDescriptions.describe.long'));
        break;
      case 'shortlog':
        if (shortlogCount && shortlogCount !== 10) descriptions.push(t('gitCommands.paramDescriptions.shortlog.count'));
        if (shortlogEmail) descriptions.push(t('gitCommands.paramDescriptions.shortlog.email'));
        if (shortlogGroup === 'committer') descriptions.push(t('gitCommands.paramDescriptions.shortlog.committer'));
        break;
      case 'switch':
        if (switchCreate) descriptions.push(t('gitCommands.paramDescriptions.switch.create'));
        if (switchTrack) descriptions.push(t('gitCommands.paramDescriptions.switch.track'));
        if (switchDetach) descriptions.push(t('gitCommands.paramDescriptions.switch.detach'));
        break;
      case 'restore':
        if (restoreStaged) descriptions.push(t('gitCommands.paramDescriptions.restore.staged'));
        if (restoreWorktree) descriptions.push(t('gitCommands.paramDescriptions.restore.worktree'));
        if (restoreSource) descriptions.push(t('gitCommands.paramDescriptions.restore.source'));
        break;
      case 'apply':
        if (applyCheck) descriptions.push(t('gitCommands.paramDescriptions.apply.check'));
        if (applyReverse) descriptions.push(t('gitCommands.paramDescriptions.apply.reverse'));
        if (apply3way) descriptions.push(t('gitCommands.paramDescriptions.apply.3way'));
        break;
      case 'format-patch':
        if (formatPatchNumbered) descriptions.push(t('gitCommands.paramDescriptions.formatPatch.numbered'));
        if (formatPatchCoverLetter) descriptions.push(t('gitCommands.paramDescriptions.formatPatch.coverLetter'));
        if (formatPatchOutput) descriptions.push(t('gitCommands.paramDescriptions.formatPatch.output'));
        break;
      case 'submodule':
        if (submoduleRecursive) descriptions.push(t('gitCommands.paramDescriptions.submodule.recursive'));
        if (submodulePath) descriptions.push(t('gitCommands.paramDescriptions.submodule.path'));
        if (submoduleUrl) descriptions.push(t('gitCommands.paramDescriptions.submodule.url'));
        break;
      case 'worktree':
        // worktree 命令的参数说明可以在这里添加
        break;
      case 'maintenance':
        if (maintenanceTask) descriptions.push(t(`gitCommands.paramDescriptions.maintenance.${maintenanceTask}`));
        break;
    }
    
    return descriptions;
  }

  async function copyCommand() {
    if (!generatedCommand) return;
    
    try {
      await navigator.clipboard.writeText(generatedCommand);
      copied = true;
      setTimeout(() => {
        copied = false;
      }, 2000);
    } catch (error) {
      console.error('Failed to copy:', error);
    }
  }

  function clear() {
    commitMessage = '';
    pushBranch = '';
    pullBranch = '';
    branchName = '';
    branchOldName = '';
    mergeBranch = '';
    tagName = '';
    tagMessage = '';
    logAuthor = '';
    logSearch = '';
    cloneUrl = '';
    cloneDirectory = '';
    cloneBranch = '';
    remoteUrl = '';
    stashMessage = '';
    resetTarget = 'HEAD';
    fetchRemote = 'origin';
    fetchBranch = '';
    checkoutTarget = '';
    checkoutBranch = '';
    checkoutFile = '';
    addFiles = '';
    diffFile = '';
    diffCommit1 = '';
    diffCommit2 = '';
    showCommit = 'HEAD';
    showFile = '';
    revertCommit = '';
    cherryPickCommit = '';
    rebaseBranch = '';
    rebaseOnto = '';
    configKey = '';
    configValue = '';
    initTemplate = '';
    initDirectory = '';
    rmFiles = '';
    mvSource = '';
    mvDestination = '';
    submodulePath = '';
    submoduleUrl = '';
    worktreePath = '';
    worktreeBranch = '';
    blameFile = '';
    blameLineStart = '';
    blameLineEnd = '';
    grepPattern = '';
    grepFile = '';
    bisectCommit = '';
    bisectScript = '';
    reflogRef = '';
    archiveOutput = '';
    archiveTree = 'HEAD';
    archivePrefix = '';
    describeCommit = 'HEAD';
    switchBranch = '';
    restoreSource = '';
    restoreFile = '';
    applyPatch = '';
    formatPatchRange = '';
    formatPatchOutput = '';
    notesCommit = 'HEAD';
    notesMessage = '';
    bundleFile = '';
    bundleBranch = '';
    sparseCheckoutPaths = '';
    generatedCommand = '';
  }

  // 命令类型变化时自动生成
  $effect(() => {
    commandType;
    generateCommand();
  });

  // 相关字段变化时自动生成
  $effect(() => {
    commitMessage;
    commitAll;
    commitAmend;
    pushRemote;
    pushBranch;
    pushForce;
    pushTags;
    pullRemote;
    pullBranch;
    pullRebase;
    fetchRemote;
    fetchBranch;
    fetchAll;
    fetchPrune;
    fetchTags;
    branchName;
    branchOperation;
    branchOldName;
    branchCheckout;
    checkoutTarget;
    checkoutBranch;
    checkoutCreate;
    checkoutFile;
    addFiles;
    addAll;
    addPatch;
    addUpdate;
    diffFile;
    diffStaged;
    diffCached;
    diffCommit1;
    diffCommit2;
    diffStat;
    showCommit;
    showFile;
    showStat;
    showNameOnly;
    revertCommit;
    revertNoCommit;
    revertNoEdit;
    cherryPickCommit;
    cherryPickNoCommit;
    cherryPickEdit;
    mergeBranch;
    mergeNoFF;
    mergeSquash;
    rebaseBranch;
    rebaseInteractive;
    rebaseOnto;
    rebaseContinue;
    rebaseAbort;
    tagName;
    tagMessage;
    tagOperation;
    tagAnnotated;
    logCount;
    logGraph;
    logOneline;
    logAuthor;
    logSearch;
    statusShort;
    cloneUrl;
    cloneDirectory;
    cloneDepth;
    cloneBranch;
    remoteName;
    remoteUrl;
    remoteOperation;
    stashMessage;
    stashOperation;
    resetMode;
    resetTarget;
    configKey;
    configValue;
    configOperation;
    configGlobal;
    configLocal;
    initBare;
    initTemplate;
    initDirectory;
    rmFiles;
    rmCached;
    rmRecursive;
    rmForce;
    mvSource;
    mvDestination;
    cleanDryRun;
    cleanForce;
    cleanInteractive;
    cleanDirectory;
    submodulePath;
    submoduleOperation;
    submoduleUrl;
    submoduleRecursive;
    worktreePath;
    worktreeBranch;
    worktreeOperation;
    blameFile;
    blameLineStart;
    blameLineEnd;
    blameShowEmail;
    blameShowLineNumbers;
    grepPattern;
    grepFile;
    grepCaseInsensitive;
    grepRecursive;
    grepShowLineNumbers;
    grepExtendedRegex;
    bisectOperation;
    bisectCommit;
    bisectScript;
    reflogShowAll;
    reflogCount;
    reflogRef;
    archiveFormat;
    archiveOutput;
    archiveTree;
    archivePrefix;
    describeCommit;
    describeTags;
    describeAll;
    describeLong;
    shortlogCount;
    shortlogEmail;
    shortlogGroup;
    switchBranch;
    switchCreate;
    switchTrack;
    switchDetach;
    restoreSource;
    restoreStaged;
    restoreWorktree;
    restoreFile;
    applyPatch;
    applyCheck;
    applyReverse;
    apply3way;
    formatPatchRange;
    formatPatchOutput;
    formatPatchNumbered;
    formatPatchCoverLetter;
    notesOperation;
    notesCommit;
    notesMessage;
    notesRef;
    bundleOperation;
    bundleFile;
    bundleBranch;
    bundleAll;
    sparseCheckoutOperation;
    sparseCheckoutPaths;
    sparseCheckoutCone;
    maintenanceOperation;
    maintenanceTask;
    generateCommand();
  });
</script>

<div class="flex flex-col h-full w-full p-2">
  <div class="card flex-1 flex flex-col space-y-6">
    {#if generatedCommand}
      <div class="flex items-center justify-end">
        <div class="flex items-center gap-2">
          <button
            onclick={copyCommand}
            class="btn-secondary text-sm transition-all duration-200 {copied ? 'bg-green-500 hover:bg-green-600 text-white' : ''}"
          >
            {#if copied}
              <Check class="w-4 h-4 inline mr-1" />
              {t('common.copied')}
            {:else}
              <Copy class="w-4 h-4 inline mr-1" />
              {t('common.copy')}
            {/if}
          </button>
          <button type="button" class="btn-secondary text-sm" onclick={clear}>
            <Trash2 class="w-4 h-4 inline mr-1" />
            {t('gitCommands.clear')}
          </button>
        </div>
      </div>
    {/if}

    <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
      <!-- 左侧：命令类型和参数 -->
      <div class="space-y-4">
        <div>
          <label for="command-type" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
            {t('gitCommands.commandType')}
          </label>
          <select id="command-type" bind:value={commandType} class="input">
            <option value="commit">{t('gitCommands.types.commit')}</option>
            <option value="add">{t('gitCommands.types.add')}</option>
            <option value="push">{t('gitCommands.types.push')}</option>
            <option value="pull">{t('gitCommands.types.pull')}</option>
            <option value="fetch">{t('gitCommands.types.fetch')}</option>
            <option value="branch">{t('gitCommands.types.branch')}</option>
            <option value="checkout">{t('gitCommands.types.checkout')}</option>
            <option value="merge">{t('gitCommands.types.merge')}</option>
            <option value="rebase">{t('gitCommands.types.rebase')}</option>
            <option value="tag">{t('gitCommands.types.tag')}</option>
            <option value="log">{t('gitCommands.types.log')}</option>
            <option value="status">{t('gitCommands.types.status')}</option>
            <option value="diff">{t('gitCommands.types.diff')}</option>
            <option value="show">{t('gitCommands.types.show')}</option>
            <option value="revert">{t('gitCommands.types.revert')}</option>
            <option value="cherry-pick">{t('gitCommands.types.cherryPick')}</option>
            <option value="clone">{t('gitCommands.types.clone')}</option>
            <option value="init">{t('gitCommands.types.init')}</option>
            <option value="remote">{t('gitCommands.types.remote')}</option>
            <option value="stash">{t('gitCommands.types.stash')}</option>
            <option value="reset">{t('gitCommands.types.reset')}</option>
            <option value="rm">{t('gitCommands.types.rm')}</option>
            <option value="mv">{t('gitCommands.types.mv')}</option>
            <option value="clean">{t('gitCommands.types.clean')}</option>
            <option value="config">{t('gitCommands.types.config')}</option>
            <option value="submodule">{t('gitCommands.types.submodule')}</option>
            <option value="worktree">{t('gitCommands.types.worktree')}</option>
            <option value="blame">{t('gitCommands.types.blame')}</option>
            <option value="grep">{t('gitCommands.types.grep')}</option>
            <option value="bisect">{t('gitCommands.types.bisect')}</option>
            <option value="reflog">{t('gitCommands.types.reflog')}</option>
            <option value="archive">{t('gitCommands.types.archive')}</option>
            <option value="describe">{t('gitCommands.types.describe')}</option>
            <option value="shortlog">{t('gitCommands.types.shortlog')}</option>
            <option value="switch">{t('gitCommands.types.switch')}</option>
            <option value="restore">{t('gitCommands.types.restore')}</option>
            <option value="apply">{t('gitCommands.types.apply')}</option>
            <option value="format-patch">{t('gitCommands.types.formatPatch')}</option>
            <option value="notes">{t('gitCommands.types.notes')}</option>
            <option value="bundle">{t('gitCommands.types.bundle')}</option>
            <option value="sparse-checkout">{t('gitCommands.types.sparseCheckout')}</option>
            <option value="maintenance">{t('gitCommands.types.maintenance')}</option>
          </select>
        </div>

        <!-- Commit 参数 -->
        {#if commandType === 'commit'}
          <div class="space-y-3">
            <div>
              <label for="commit-message" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.commit.message')}
              </label>
              <input
                id="commit-message"
                type="text"
                bind:value={commitMessage}
                placeholder={t('gitCommands.commit.messagePlaceholder')}
                class="input"
              />
            </div>
            <div class="flex items-center gap-4">
              <label class="flex items-center">
                <input type="checkbox" bind:checked={commitAll} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.commit.addAll')}</span>
              </label>
              <label class="flex items-center">
                <input type="checkbox" bind:checked={commitAmend} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.commit.amend')}</span>
              </label>
            </div>
          </div>
        {/if}

        <!-- Push 参数 -->
        {#if commandType === 'push'}
          <div class="space-y-3">
            <div>
              <label for="push-remote" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.push.remote')}
              </label>
              <input
                id="push-remote"
                type="text"
                bind:value={pushRemote}
                placeholder="origin"
                class="input"
              />
            </div>
            <div>
              <label for="push-branch" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.push.branch')}
              </label>
              <input
                id="push-branch"
                type="text"
                bind:value={pushBranch}
                placeholder={t('gitCommands.push.branchPlaceholder')}
                class="input"
              />
            </div>
            <div class="flex items-center gap-4">
              <label class="flex items-center">
                <input type="checkbox" bind:checked={pushForce} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.push.force')}</span>
              </label>
              <label class="flex items-center">
                <input type="checkbox" bind:checked={pushTags} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.push.tags')}</span>
              </label>
            </div>
          </div>
        {/if}

        <!-- Pull 参数 -->
        {#if commandType === 'pull'}
          <div class="space-y-3">
            <div>
              <label for="pull-remote" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.pull.remote')}
              </label>
              <input
                id="pull-remote"
                type="text"
                bind:value={pullRemote}
                placeholder="origin"
                class="input"
              />
            </div>
            <div>
              <label for="pull-branch" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.pull.branch')}
              </label>
              <input
                id="pull-branch"
                type="text"
                bind:value={pullBranch}
                placeholder={t('gitCommands.pull.branchPlaceholder')}
                class="input"
              />
            </div>
            <label class="flex items-center">
              <input type="checkbox" bind:checked={pullRebase} class="mr-2" />
              <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.pull.rebase')}</span>
            </label>
          </div>
        {/if}

        <!-- Branch 参数 -->
        {#if commandType === 'branch'}
          <div class="space-y-3">
            <div>
              <label for="branch-operation" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.branch.operation')}
              </label>
              <select id="branch-operation" bind:value={branchOperation} class="input">
                <option value="create">{t('gitCommands.branch.create')}</option>
                <option value="checkout">{t('gitCommands.branch.checkout')}</option>
                <option value="delete">{t('gitCommands.branch.delete')}</option>
                <option value="rename">{t('gitCommands.branch.rename')}</option>
                <option value="list">{t('gitCommands.branch.list')}</option>
              </select>
            </div>
            {#if branchOperation !== 'list'}
              {#if branchOperation === 'rename'}
                <div>
                  <label for="branch-old-name" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                    {t('gitCommands.branch.oldName')}
                  </label>
                  <input
                    id="branch-old-name"
                    type="text"
                    bind:value={branchOldName}
                    placeholder={t('gitCommands.branch.oldNamePlaceholder')}
                    class="input"
                  />
                </div>
              {/if}
              <div>
                <label for="branch-name" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                  {t('gitCommands.branch.name')}
                </label>
                <input
                  id="branch-name"
                  type="text"
                  bind:value={branchName}
                  placeholder={t('gitCommands.branch.namePlaceholder')}
                  class="input"
                />
              </div>
              {#if branchOperation === 'create'}
                <label class="flex items-center">
                  <input type="checkbox" bind:checked={branchCheckout} class="mr-2" />
                  <span class="text-sm text-gray-700 dark:text-gray-300">
                    {t('gitCommands.branch.checkout')}
                  </span>
                </label>
              {/if}
            {:else}
              <label class="flex items-center">
                <input type="checkbox" bind:checked={branchCheckout} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">
                  {t('gitCommands.branch.showAll')}
                </span>
              </label>
            {/if}
          </div>
        {/if}

        <!-- Merge 参数 -->
        {#if commandType === 'merge'}
          <div class="space-y-3">
            <div>
              <label for="merge-branch" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.merge.branch')}
              </label>
              <input
                id="merge-branch"
                type="text"
                bind:value={mergeBranch}
                placeholder={t('gitCommands.merge.branchPlaceholder')}
                class="input"
              />
            </div>
            <div class="flex items-center gap-4">
              <label class="flex items-center">
                <input type="checkbox" bind:checked={mergeNoFF} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.merge.noFF')}</span>
              </label>
              <label class="flex items-center">
                <input type="checkbox" bind:checked={mergeSquash} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.merge.squash')}</span>
              </label>
            </div>
          </div>
        {/if}

        <!-- Tag 参数 -->
        {#if commandType === 'tag'}
          <div class="space-y-3">
            <div>
              <label for="tag-operation" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.tag.operation')}
              </label>
              <select id="tag-operation" bind:value={tagOperation} class="input">
                <option value="create">{t('gitCommands.tag.create')}</option>
                <option value="delete">{t('gitCommands.tag.delete')}</option>
                <option value="list">{t('gitCommands.tag.list')}</option>
              </select>
            </div>
            {#if tagOperation !== 'list'}
              <div>
                <label for="tag-name" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                  {t('gitCommands.tag.name')}
                </label>
                <input
                  id="tag-name"
                  type="text"
                  bind:value={tagName}
                  placeholder={t('gitCommands.tag.namePlaceholder')}
                  class="input"
                />
              </div>
              {#if tagOperation === 'create'}
                <label class="flex items-center">
                  <input type="checkbox" bind:checked={tagAnnotated} class="mr-2" />
                  <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.tag.annotated')}</span>
                </label>
                {#if tagAnnotated}
                  <div>
                    <label for="tag-message" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                      {t('gitCommands.tag.message')}
                    </label>
                    <input
                      id="tag-message"
                      type="text"
                      bind:value={tagMessage}
                      placeholder={t('gitCommands.tag.messagePlaceholder')}
                      class="input"
                    />
                  </div>
                {/if}
              {/if}
            {/if}
          </div>
        {/if}

        <!-- Log 参数 -->
        {#if commandType === 'log'}
          <div class="space-y-3">
            <div>
              <label for="log-count" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.log.count')}
              </label>
              <input
                id="log-count"
                type="number"
                bind:value={logCount}
                min="1"
                max="1000"
                class="input"
              />
            </div>
            <div class="flex items-center gap-4">
              <label class="flex items-center">
                <input type="checkbox" bind:checked={logGraph} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.log.graph')}</span>
              </label>
              <label class="flex items-center">
                <input type="checkbox" bind:checked={logOneline} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.log.oneline')}</span>
              </label>
            </div>
            <div>
              <label for="log-author" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.log.author')}
              </label>
              <input
                id="log-author"
                type="text"
                bind:value={logAuthor}
                placeholder={t('gitCommands.log.authorPlaceholder')}
                class="input"
              />
            </div>
            <div>
              <label for="log-search" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.log.search')}
              </label>
              <input
                id="log-search"
                type="text"
                bind:value={logSearch}
                placeholder={t('gitCommands.log.searchPlaceholder')}
                class="input"
              />
            </div>
          </div>
        {/if}

        <!-- Status 参数 -->
        {#if commandType === 'status'}
          <div>
            <label class="flex items-center">
              <input type="checkbox" bind:checked={statusShort} class="mr-2" />
              <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.status.short')}</span>
            </label>
          </div>
        {/if}

        <!-- Clone 参数 -->
        {#if commandType === 'clone'}
          <div class="space-y-3">
            <div>
              <label for="clone-url" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.clone.url')}
              </label>
              <input
                id="clone-url"
                type="text"
                bind:value={cloneUrl}
                placeholder={t('gitCommands.clone.urlPlaceholder')}
                class="input"
              />
            </div>
            <div>
              <label for="clone-directory" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.clone.directory')}
              </label>
              <input
                id="clone-directory"
                type="text"
                bind:value={cloneDirectory}
                placeholder={t('gitCommands.clone.directoryPlaceholder')}
                class="input"
              />
            </div>
            <div>
              <label for="clone-branch" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.clone.branch')}
              </label>
              <input
                id="clone-branch"
                type="text"
                bind:value={cloneBranch}
                placeholder={t('gitCommands.clone.branchPlaceholder')}
                class="input"
              />
            </div>
            <div>
              <label for="clone-depth" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.clone.depth')}
              </label>
              <input
                id="clone-depth"
                type="number"
                bind:value={cloneDepth}
                placeholder={t('gitCommands.clone.depthPlaceholder')}
                class="input"
              />
            </div>
          </div>
        {/if}

        <!-- Remote 参数 -->
        {#if commandType === 'remote'}
          <div class="space-y-3">
            <div>
              <label for="remote-operation" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.remote.operation')}
              </label>
              <select id="remote-operation" bind:value={remoteOperation} class="input">
                <option value="add">{t('gitCommands.remote.add')}</option>
                <option value="remove">{t('gitCommands.remote.remove')}</option>
                <option value="set-url">{t('gitCommands.remote.setUrl')}</option>
                <option value="list">{t('gitCommands.remote.list')}</option>
              </select>
            </div>
            {#if remoteOperation !== 'list'}
              <div>
                <label for="remote-name" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                  {t('gitCommands.remote.name')}
                </label>
                <input
                  id="remote-name"
                  type="text"
                  bind:value={remoteName}
                  placeholder="origin"
                  class="input"
                />
              </div>
              {#if remoteOperation === 'add' || remoteOperation === 'set-url'}
                <div>
                  <label for="remote-url" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                    {t('gitCommands.remote.url')}
                  </label>
                  <input
                    id="remote-url"
                    type="text"
                    bind:value={remoteUrl}
                    placeholder={t('gitCommands.remote.urlPlaceholder')}
                    class="input"
                  />
                </div>
              {/if}
            {/if}
          </div>
        {/if}

        <!-- Stash 参数 -->
        {#if commandType === 'stash'}
          <div class="space-y-3">
            <div>
              <label for="stash-operation" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.stash.operation')}
              </label>
              <select id="stash-operation" bind:value={stashOperation} class="input">
                <option value="save">{t('gitCommands.stash.save')}</option>
                <option value="list">{t('gitCommands.stash.list')}</option>
                <option value="pop">{t('gitCommands.stash.pop')}</option>
                <option value="apply">{t('gitCommands.stash.apply')}</option>
                <option value="drop">{t('gitCommands.stash.drop')}</option>
              </select>
            </div>
            {#if stashOperation === 'save'}
              <div>
                <label for="stash-message" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                  {t('gitCommands.stash.message')}
                </label>
                <input
                  id="stash-message"
                  type="text"
                  bind:value={stashMessage}
                  placeholder={t('gitCommands.stash.messagePlaceholder')}
                  class="input"
                />
              </div>
            {/if}
          </div>
        {/if}

        <!-- Reset 参数 -->
        {#if commandType === 'reset'}
          <div class="space-y-3">
            <div>
              <label for="reset-mode" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.reset.mode')}
              </label>
              <select id="reset-mode" bind:value={resetMode} class="input">
                <option value="soft">{t('gitCommands.reset.soft')}</option>
                <option value="mixed">{t('gitCommands.reset.mixed')}</option>
                <option value="hard">{t('gitCommands.reset.hard')}</option>
              </select>
            </div>
            <div>
              <label for="reset-target" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.reset.target')}
              </label>
              <input
                id="reset-target"
                type="text"
                bind:value={resetTarget}
                placeholder="HEAD"
                class="input"
              />
            </div>
          </div>
        {/if}

        <!-- Fetch 参数 -->
        {#if commandType === 'fetch'}
          <div class="space-y-3">
            <div>
              <label for="fetch-remote" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.fetch.remote')}
              </label>
              <input
                id="fetch-remote"
                type="text"
                bind:value={fetchRemote}
                placeholder="origin"
                class="input"
              />
            </div>
            <div>
              <label for="fetch-branch" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.fetch.branch')}
              </label>
              <input
                id="fetch-branch"
                type="text"
                bind:value={fetchBranch}
                placeholder={t('gitCommands.fetch.branchPlaceholder')}
                class="input"
              />
            </div>
            <div class="flex items-center gap-4">
              <label class="flex items-center">
                <input type="checkbox" bind:checked={fetchAll} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.fetch.all')}</span>
              </label>
              <label class="flex items-center">
                <input type="checkbox" bind:checked={fetchPrune} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.fetch.prune')}</span>
              </label>
              <label class="flex items-center">
                <input type="checkbox" bind:checked={fetchTags} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.fetch.tags')}</span>
              </label>
            </div>
          </div>
        {/if}

        <!-- Checkout 参数 -->
        {#if commandType === 'checkout'}
          <div class="space-y-3">
            <div>
              <label for="checkout-target" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.checkout.target')}
              </label>
              <input
                id="checkout-target"
                type="text"
                bind:value={checkoutTarget}
                placeholder={t('gitCommands.checkout.targetPlaceholder')}
                class="input"
              />
            </div>
            <div>
              <label for="checkout-branch" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.checkout.branch')}
              </label>
              <input
                id="checkout-branch"
                type="text"
                bind:value={checkoutBranch}
                placeholder={t('gitCommands.checkout.branchPlaceholder')}
                class="input"
              />
            </div>
            <div>
              <label for="checkout-file" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.checkout.file')}
              </label>
              <input
                id="checkout-file"
                type="text"
                bind:value={checkoutFile}
                placeholder={t('gitCommands.checkout.filePlaceholder')}
                class="input"
              />
            </div>
            <label class="flex items-center">
              <input type="checkbox" bind:checked={checkoutCreate} class="mr-2" />
              <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.checkout.create')}</span>
            </label>
          </div>
        {/if}

        <!-- Add 参数 -->
        {#if commandType === 'add'}
          <div class="space-y-3">
            <div>
              <label for="add-files" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.add.files')}
              </label>
              <input
                id="add-files"
                type="text"
                bind:value={addFiles}
                placeholder={t('gitCommands.add.filesPlaceholder')}
                class="input"
              />
            </div>
            <div class="flex items-center gap-4">
              <label class="flex items-center">
                <input type="checkbox" bind:checked={addAll} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.add.all')}</span>
              </label>
              <label class="flex items-center">
                <input type="checkbox" bind:checked={addPatch} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.add.patch')}</span>
              </label>
              <label class="flex items-center">
                <input type="checkbox" bind:checked={addUpdate} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.add.update')}</span>
              </label>
            </div>
          </div>
        {/if}

        <!-- Diff 参数 -->
        {#if commandType === 'diff'}
          <div class="space-y-3">
            <div>
              <label for="diff-file" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.diff.file')}
              </label>
              <input
                id="diff-file"
                type="text"
                bind:value={diffFile}
                placeholder={t('gitCommands.diff.filePlaceholder')}
                class="input"
              />
            </div>
            <div>
              <label for="diff-commit1" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.diff.commit1')}
              </label>
              <input
                id="diff-commit1"
                type="text"
                bind:value={diffCommit1}
                placeholder={t('gitCommands.diff.commit1Placeholder')}
                class="input"
              />
            </div>
            <div>
              <label for="diff-commit2" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.diff.commit2')}
              </label>
              <input
                id="diff-commit2"
                type="text"
                bind:value={diffCommit2}
                placeholder={t('gitCommands.diff.commit2Placeholder')}
                class="input"
              />
            </div>
            <div class="flex items-center gap-4">
              <label class="flex items-center">
                <input type="checkbox" bind:checked={diffStaged} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.diff.staged')}</span>
              </label>
              <label class="flex items-center">
                <input type="checkbox" bind:checked={diffStat} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.diff.stat')}</span>
              </label>
            </div>
          </div>
        {/if}

        <!-- Show 参数 -->
        {#if commandType === 'show'}
          <div class="space-y-3">
            <div>
              <label for="show-commit" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.show.commit')}
              </label>
              <input
                id="show-commit"
                type="text"
                bind:value={showCommit}
                placeholder="HEAD"
                class="input"
              />
            </div>
            <div>
              <label for="show-file" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.show.file')}
              </label>
              <input
                id="show-file"
                type="text"
                bind:value={showFile}
                placeholder={t('gitCommands.show.filePlaceholder')}
                class="input"
              />
            </div>
            <div class="flex items-center gap-4">
              <label class="flex items-center">
                <input type="checkbox" bind:checked={showStat} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.show.stat')}</span>
              </label>
              <label class="flex items-center">
                <input type="checkbox" bind:checked={showNameOnly} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.show.nameOnly')}</span>
              </label>
            </div>
          </div>
        {/if}

        <!-- Revert 参数 -->
        {#if commandType === 'revert'}
          <div class="space-y-3">
            <div>
              <label for="revert-commit" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.revert.commit')}
              </label>
              <input
                id="revert-commit"
                type="text"
                bind:value={revertCommit}
                placeholder={t('gitCommands.revert.commitPlaceholder')}
                class="input"
              />
            </div>
            <div class="flex items-center gap-4">
              <label class="flex items-center">
                <input type="checkbox" bind:checked={revertNoCommit} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.revert.noCommit')}</span>
              </label>
              <label class="flex items-center">
                <input type="checkbox" bind:checked={revertNoEdit} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.revert.noEdit')}</span>
              </label>
            </div>
          </div>
        {/if}

        <!-- Cherry-pick 参数 -->
        {#if commandType === 'cherry-pick'}
          <div class="space-y-3">
            <div>
              <label for="cherry-pick-commit" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.cherryPick.commit')}
              </label>
              <input
                id="cherry-pick-commit"
                type="text"
                bind:value={cherryPickCommit}
                placeholder={t('gitCommands.cherryPick.commitPlaceholder')}
                class="input"
              />
            </div>
            <div class="flex items-center gap-4">
              <label class="flex items-center">
                <input type="checkbox" bind:checked={cherryPickNoCommit} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.cherryPick.noCommit')}</span>
              </label>
              <label class="flex items-center">
                <input type="checkbox" bind:checked={cherryPickEdit} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.cherryPick.edit')}</span>
              </label>
            </div>
          </div>
        {/if}

        <!-- Rebase 参数 -->
        {#if commandType === 'rebase'}
          <div class="space-y-3">
            <div>
              <label for="rebase-branch" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.rebase.branch')}
              </label>
              <input
                id="rebase-branch"
                type="text"
                bind:value={rebaseBranch}
                placeholder={t('gitCommands.rebase.branchPlaceholder')}
                class="input"
              />
            </div>
            <div>
              <label for="rebase-onto" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.rebase.onto')}
              </label>
              <input
                id="rebase-onto"
                type="text"
                bind:value={rebaseOnto}
                placeholder={t('gitCommands.rebase.ontoPlaceholder')}
                class="input"
              />
            </div>
            <div class="flex items-center gap-4">
              <label class="flex items-center">
                <input type="checkbox" bind:checked={rebaseInteractive} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.rebase.interactive')}</span>
              </label>
              <label class="flex items-center">
                <input type="checkbox" bind:checked={rebaseContinue} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.rebase.continue')}</span>
              </label>
              <label class="flex items-center">
                <input type="checkbox" bind:checked={rebaseAbort} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.rebase.abort')}</span>
              </label>
            </div>
          </div>
        {/if}

        <!-- Config 参数 -->
        {#if commandType === 'config'}
          <div class="space-y-3">
            <div>
              <label for="config-operation" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.config.operation')}
              </label>
              <select id="config-operation" bind:value={configOperation} class="input">
                <option value="get">{t('gitCommands.config.get')}</option>
                <option value="set">{t('gitCommands.config.set')}</option>
                <option value="unset">{t('gitCommands.config.unset')}</option>
                <option value="list">{t('gitCommands.config.list')}</option>
              </select>
            </div>
            {#if configOperation !== 'list'}
              <div>
                <label for="config-key" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                  {t('gitCommands.config.key')}
                </label>
                <input
                  id="config-key"
                  type="text"
                  bind:value={configKey}
                  placeholder={t('gitCommands.config.keyPlaceholder')}
                  class="input"
                />
              </div>
              {#if configOperation === 'set'}
                <div>
                  <label for="config-value" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                    {t('gitCommands.config.value')}
                  </label>
                  <input
                    id="config-value"
                    type="text"
                    bind:value={configValue}
                    placeholder={t('gitCommands.config.valuePlaceholder')}
                    class="input"
                  />
                </div>
              {/if}
            {/if}
            <div class="flex items-center gap-4">
              <label class="flex items-center">
                <input type="checkbox" bind:checked={configGlobal} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.config.global')}</span>
              </label>
              <label class="flex items-center">
                <input type="checkbox" bind:checked={configLocal} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.config.local')}</span>
              </label>
            </div>
          </div>
        {/if}

        <!-- Init 参数 -->
        {#if commandType === 'init'}
          <div class="space-y-3">
            <div>
              <label for="init-directory" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.init.directory')}
              </label>
              <input
                id="init-directory"
                type="text"
                bind:value={initDirectory}
                placeholder={t('gitCommands.init.directoryPlaceholder')}
                class="input"
              />
            </div>
            <div>
              <label for="init-template" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.init.template')}
              </label>
              <input
                id="init-template"
                type="text"
                bind:value={initTemplate}
                placeholder={t('gitCommands.init.templatePlaceholder')}
                class="input"
              />
            </div>
            <label class="flex items-center">
              <input type="checkbox" bind:checked={initBare} class="mr-2" />
              <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.init.bare')}</span>
            </label>
          </div>
        {/if}

        <!-- Rm 参数 -->
        {#if commandType === 'rm'}
          <div class="space-y-3">
            <div>
              <label for="rm-files" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.rm.files')}
              </label>
              <input
                id="rm-files"
                type="text"
                bind:value={rmFiles}
                placeholder={t('gitCommands.rm.filesPlaceholder')}
                class="input"
              />
            </div>
            <div class="flex items-center gap-4">
              <label class="flex items-center">
                <input type="checkbox" bind:checked={rmCached} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.rm.cached')}</span>
              </label>
              <label class="flex items-center">
                <input type="checkbox" bind:checked={rmRecursive} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.rm.recursive')}</span>
              </label>
              <label class="flex items-center">
                <input type="checkbox" bind:checked={rmForce} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.rm.force')}</span>
              </label>
            </div>
          </div>
        {/if}

        <!-- Mv 参数 -->
        {#if commandType === 'mv'}
          <div class="space-y-3">
            <div>
              <label for="mv-source" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.mv.source')}
              </label>
              <input
                id="mv-source"
                type="text"
                bind:value={mvSource}
                placeholder={t('gitCommands.mv.sourcePlaceholder')}
                class="input"
              />
            </div>
            <div>
              <label for="mv-destination" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.mv.destination')}
              </label>
              <input
                id="mv-destination"
                type="text"
                bind:value={mvDestination}
                placeholder={t('gitCommands.mv.destinationPlaceholder')}
                class="input"
              />
            </div>
          </div>
        {/if}

        <!-- Clean 参数 -->
        {#if commandType === 'clean'}
          <div class="space-y-3">
            <div class="flex items-center gap-4">
              <label class="flex items-center">
                <input type="checkbox" bind:checked={cleanDryRun} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.clean.dryRun')}</span>
              </label>
              <label class="flex items-center">
                <input type="checkbox" bind:checked={cleanForce} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.clean.force')}</span>
              </label>
              <label class="flex items-center">
                <input type="checkbox" bind:checked={cleanInteractive} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.clean.interactive')}</span>
              </label>
              <label class="flex items-center">
                <input type="checkbox" bind:checked={cleanDirectory} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.clean.directory')}</span>
              </label>
            </div>
          </div>
        {/if}

        <!-- Submodule 参数 -->
        {#if commandType === 'submodule'}
          <div class="space-y-3">
            <div>
              <label for="submodule-operation" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.submodule.operation')}
              </label>
              <select id="submodule-operation" bind:value={submoduleOperation} class="input">
                <option value="add">{t('gitCommands.submodule.add')}</option>
                <option value="update">{t('gitCommands.submodule.update')}</option>
                <option value="init">{t('gitCommands.submodule.init')}</option>
                <option value="deinit">{t('gitCommands.submodule.deinit')}</option>
                <option value="status">{t('gitCommands.submodule.status')}</option>
              </select>
            </div>
            {#if submoduleOperation === 'add'}
              <div>
                <label for="submodule-url" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                  {t('gitCommands.submodule.url')}
                </label>
                <input
                  id="submodule-url"
                  type="text"
                  bind:value={submoduleUrl}
                  placeholder={t('gitCommands.submodule.urlPlaceholder')}
                  class="input"
                />
              </div>
            {/if}
            {#if submoduleOperation !== 'status'}
              <div>
                <label for="submodule-path" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                  {t('gitCommands.submodule.path')}
                </label>
                <input
                  id="submodule-path"
                  type="text"
                  bind:value={submodulePath}
                  placeholder={t('gitCommands.submodule.pathPlaceholder')}
                  class="input"
                />
              </div>
            {/if}
            {#if submoduleOperation === 'update'}
              <label class="flex items-center">
                <input type="checkbox" bind:checked={submoduleRecursive} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.submodule.recursive')}</span>
              </label>
            {/if}
          </div>
        {/if}

        <!-- Worktree 参数 -->
        {#if commandType === 'worktree'}
          <div class="space-y-3">
            <div>
              <label for="worktree-operation" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.worktree.operation')}
              </label>
              <select id="worktree-operation" bind:value={worktreeOperation} class="input">
                <option value="add">{t('gitCommands.worktree.add')}</option>
                <option value="list">{t('gitCommands.worktree.list')}</option>
                <option value="remove">{t('gitCommands.worktree.remove')}</option>
                <option value="prune">{t('gitCommands.worktree.prune')}</option>
              </select>
            </div>
            {#if worktreeOperation === 'add'}
              <div>
                <label for="worktree-path" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                  {t('gitCommands.worktree.path')}
                </label>
                <input
                  id="worktree-path"
                  type="text"
                  bind:value={worktreePath}
                  placeholder={t('gitCommands.worktree.pathPlaceholder')}
                  class="input"
                />
              </div>
              <div>
                <label for="worktree-branch" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                  {t('gitCommands.worktree.branch')}
                </label>
                <input
                  id="worktree-branch"
                  type="text"
                  bind:value={worktreeBranch}
                  placeholder={t('gitCommands.worktree.branchPlaceholder')}
                  class="input"
                />
              </div>
            {:else if worktreeOperation === 'remove'}
              <div>
                <label for="worktree-path" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                  {t('gitCommands.worktree.path')}
                </label>
                <input
                  id="worktree-path"
                  type="text"
                  bind:value={worktreePath}
                  placeholder={t('gitCommands.worktree.pathPlaceholder')}
                  class="input"
                />
              </div>
            {/if}
          </div>
        {/if}

        <!-- Blame 参数 -->
        {#if commandType === 'blame'}
          <div class="space-y-3">
            <div>
              <label for="blame-file" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.blame.file')}
              </label>
              <input
                id="blame-file"
                type="text"
                bind:value={blameFile}
                placeholder={t('gitCommands.blame.filePlaceholder')}
                class="input"
              />
            </div>
            <div class="grid grid-cols-2 gap-3">
              <div>
                <label for="blame-line-start" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                  {t('gitCommands.blame.lineStart')}
                </label>
                <input
                  id="blame-line-start"
                  type="text"
                  bind:value={blameLineStart}
                  placeholder={t('gitCommands.blame.lineStartPlaceholder')}
                  class="input"
                />
              </div>
              <div>
                <label for="blame-line-end" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                  {t('gitCommands.blame.lineEnd')}
                </label>
                <input
                  id="blame-line-end"
                  type="text"
                  bind:value={blameLineEnd}
                  placeholder={t('gitCommands.blame.lineEndPlaceholder')}
                  class="input"
                />
              </div>
            </div>
            <div class="flex items-center gap-4">
              <label class="flex items-center">
                <input type="checkbox" bind:checked={blameShowEmail} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.blame.showEmail')}</span>
              </label>
              <label class="flex items-center">
                <input type="checkbox" bind:checked={blameShowLineNumbers} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.blame.showLineNumbers')}</span>
              </label>
            </div>
          </div>
        {/if}

        <!-- Grep 参数 -->
        {#if commandType === 'grep'}
          <div class="space-y-3">
            <div>
              <label for="grep-pattern" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.grep.pattern')}
              </label>
              <input
                id="grep-pattern"
                type="text"
                bind:value={grepPattern}
                placeholder={t('gitCommands.grep.patternPlaceholder')}
                class="input"
              />
            </div>
            <div>
              <label for="grep-file" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.grep.file')}
              </label>
              <input
                id="grep-file"
                type="text"
                bind:value={grepFile}
                placeholder={t('gitCommands.grep.filePlaceholder')}
                class="input"
              />
            </div>
            <div class="flex items-center gap-4 flex-wrap">
              <label class="flex items-center">
                <input type="checkbox" bind:checked={grepCaseInsensitive} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.grep.caseInsensitive')}</span>
              </label>
              <label class="flex items-center">
                <input type="checkbox" bind:checked={grepRecursive} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.grep.recursive')}</span>
              </label>
              <label class="flex items-center">
                <input type="checkbox" bind:checked={grepShowLineNumbers} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.grep.showLineNumbers')}</span>
              </label>
              <label class="flex items-center">
                <input type="checkbox" bind:checked={grepExtendedRegex} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.grep.extendedRegex')}</span>
              </label>
            </div>
          </div>
        {/if}

        <!-- Bisect 参数 -->
        {#if commandType === 'bisect'}
          <div class="space-y-3">
            <div>
              <label for="bisect-operation" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.bisect.operation')}
              </label>
              <select id="bisect-operation" bind:value={bisectOperation} class="input">
                <option value="start">{t('gitCommands.bisect.start')}</option>
                <option value="good">{t('gitCommands.bisect.good')}</option>
                <option value="bad">{t('gitCommands.bisect.bad')}</option>
                <option value="skip">{t('gitCommands.bisect.skip')}</option>
                <option value="reset">{t('gitCommands.bisect.reset')}</option>
                <option value="run">{t('gitCommands.bisect.run')}</option>
              </select>
            </div>
            {#if bisectOperation !== 'reset'}
              <div>
                <label for="bisect-commit" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                  {t('gitCommands.bisect.commit')}
                </label>
                <input
                  id="bisect-commit"
                  type="text"
                  bind:value={bisectCommit}
                  placeholder={t('gitCommands.bisect.commitPlaceholder')}
                  class="input"
                />
              </div>
            {/if}
            {#if bisectOperation === 'run'}
              <div>
                <label for="bisect-script" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                  {t('gitCommands.bisect.script')}
                </label>
                <input
                  id="bisect-script"
                  type="text"
                  bind:value={bisectScript}
                  placeholder={t('gitCommands.bisect.scriptPlaceholder')}
                  class="input"
                />
              </div>
            {/if}
          </div>
        {/if}

        <!-- Reflog 参数 -->
        {#if commandType === 'reflog'}
          <div class="space-y-3">
            <div>
              <label for="reflog-ref" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.reflog.ref')}
              </label>
              <input
                id="reflog-ref"
                type="text"
                bind:value={reflogRef}
                placeholder={t('gitCommands.reflog.refPlaceholder')}
                class="input"
              />
            </div>
            <div>
              <label for="reflog-count" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.reflog.count')}
              </label>
              <input
                id="reflog-count"
                type="number"
                bind:value={reflogCount}
                placeholder="10"
                class="input"
              />
            </div>
            <label class="flex items-center">
              <input type="checkbox" bind:checked={reflogShowAll} class="mr-2" />
              <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.reflog.showAll')}</span>
            </label>
          </div>
        {/if}

        <!-- Archive 参数 -->
        {#if commandType === 'archive'}
          <div class="space-y-3">
            <div>
              <label for="archive-format" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.archive.format')}
              </label>
              <select id="archive-format" bind:value={archiveFormat} class="input">
                <option value="tar.gz">tar.gz</option>
                <option value="tar">tar</option>
                <option value="zip">zip</option>
              </select>
            </div>
            <div>
              <label for="archive-output" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.archive.output')}
              </label>
              <input
                id="archive-output"
                type="text"
                bind:value={archiveOutput}
                placeholder={t('gitCommands.archive.outputPlaceholder')}
                class="input"
              />
            </div>
            <div>
              <label for="archive-tree" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.archive.tree')}
              </label>
              <input
                id="archive-tree"
                type="text"
                bind:value={archiveTree}
                placeholder="HEAD"
                class="input"
              />
            </div>
            <div>
              <label for="archive-prefix" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.archive.prefix')}
              </label>
              <input
                id="archive-prefix"
                type="text"
                bind:value={archivePrefix}
                placeholder={t('gitCommands.archive.prefixPlaceholder')}
                class="input"
              />
            </div>
          </div>
        {/if}

        <!-- Describe 参数 -->
        {#if commandType === 'describe'}
          <div class="space-y-3">
            <div>
              <label for="describe-commit" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.describe.commit')}
              </label>
              <input
                id="describe-commit"
                type="text"
                bind:value={describeCommit}
                placeholder="HEAD"
                class="input"
              />
            </div>
            <div class="flex items-center gap-4">
              <label class="flex items-center">
                <input type="checkbox" bind:checked={describeTags} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.describe.tags')}</span>
              </label>
              <label class="flex items-center">
                <input type="checkbox" bind:checked={describeAll} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.describe.all')}</span>
              </label>
              <label class="flex items-center">
                <input type="checkbox" bind:checked={describeLong} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.describe.long')}</span>
              </label>
            </div>
          </div>
        {/if}

        <!-- Shortlog 参数 -->
        {#if commandType === 'shortlog'}
          <div class="space-y-3">
            <div>
              <label for="shortlog-count" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.shortlog.count')}
              </label>
              <input
                id="shortlog-count"
                type="number"
                bind:value={shortlogCount}
                placeholder="10"
                class="input"
              />
            </div>
            <div>
              <label for="shortlog-group" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.shortlog.group')}
              </label>
              <select id="shortlog-group" bind:value={shortlogGroup} class="input">
                <option value="author">{t('gitCommands.shortlog.author')}</option>
                <option value="committer">{t('gitCommands.shortlog.committer')}</option>
              </select>
            </div>
            <label class="flex items-center">
              <input type="checkbox" bind:checked={shortlogEmail} class="mr-2" />
              <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.shortlog.email')}</span>
            </label>
          </div>
        {/if}

        <!-- Switch 参数 -->
        {#if commandType === 'switch'}
          <div class="space-y-3">
            <div>
              <label for="switch-branch" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.switch.branch')}
              </label>
              <input
                id="switch-branch"
                type="text"
                bind:value={switchBranch}
                placeholder={t('gitCommands.switch.branchPlaceholder')}
                class="input"
              />
            </div>
            <div class="flex items-center gap-4">
              <label class="flex items-center">
                <input type="checkbox" bind:checked={switchCreate} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.switch.create')}</span>
              </label>
              <label class="flex items-center">
                <input type="checkbox" bind:checked={switchTrack} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.switch.track')}</span>
              </label>
              <label class="flex items-center">
                <input type="checkbox" bind:checked={switchDetach} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.switch.detach')}</span>
              </label>
            </div>
          </div>
        {/if}

        <!-- Restore 参数 -->
        {#if commandType === 'restore'}
          <div class="space-y-3">
            <div>
              <label for="restore-file" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.restore.file')}
              </label>
              <input
                id="restore-file"
                type="text"
                bind:value={restoreFile}
                placeholder={t('gitCommands.restore.filePlaceholder')}
                class="input"
              />
            </div>
            <div>
              <label for="restore-source" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.restore.source')}
              </label>
              <input
                id="restore-source"
                type="text"
                bind:value={restoreSource}
                placeholder={t('gitCommands.restore.sourcePlaceholder')}
                class="input"
              />
            </div>
            <div class="flex items-center gap-4">
              <label class="flex items-center">
                <input type="checkbox" bind:checked={restoreStaged} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.restore.staged')}</span>
              </label>
              <label class="flex items-center">
                <input type="checkbox" bind:checked={restoreWorktree} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.restore.worktree')}</span>
              </label>
            </div>
          </div>
        {/if}

        <!-- Apply 参数 -->
        {#if commandType === 'apply'}
          <div class="space-y-3">
            <div>
              <label for="apply-patch" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.apply.patch')}
              </label>
              <input
                id="apply-patch"
                type="text"
                bind:value={applyPatch}
                placeholder={t('gitCommands.apply.patchPlaceholder')}
                class="input"
              />
            </div>
            <div class="flex items-center gap-4">
              <label class="flex items-center">
                <input type="checkbox" bind:checked={applyCheck} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.apply.check')}</span>
              </label>
              <label class="flex items-center">
                <input type="checkbox" bind:checked={applyReverse} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.apply.reverse')}</span>
              </label>
              <label class="flex items-center">
                <input type="checkbox" bind:checked={apply3way} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.apply.3way')}</span>
              </label>
            </div>
          </div>
        {/if}

        <!-- Format-patch 参数 -->
        {#if commandType === 'format-patch'}
          <div class="space-y-3">
            <div>
              <label for="format-patch-range" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.formatPatch.range')}
              </label>
              <input
                id="format-patch-range"
                type="text"
                bind:value={formatPatchRange}
                placeholder={t('gitCommands.formatPatch.rangePlaceholder')}
                class="input"
              />
            </div>
            <div>
              <label for="format-patch-output" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.formatPatch.output')}
              </label>
              <input
                id="format-patch-output"
                type="text"
                bind:value={formatPatchOutput}
                placeholder={t('gitCommands.formatPatch.outputPlaceholder')}
                class="input"
              />
            </div>
            <div class="flex items-center gap-4">
              <label class="flex items-center">
                <input type="checkbox" bind:checked={formatPatchNumbered} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.formatPatch.numbered')}</span>
              </label>
              <label class="flex items-center">
                <input type="checkbox" bind:checked={formatPatchCoverLetter} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.formatPatch.coverLetter')}</span>
              </label>
            </div>
          </div>
        {/if}

        <!-- Notes 参数 -->
        {#if commandType === 'notes'}
          <div class="space-y-3">
            <div>
              <label for="notes-operation" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.notes.operation')}
              </label>
              <select id="notes-operation" bind:value={notesOperation} class="input">
                <option value="add">{t('gitCommands.notes.add')}</option>
                <option value="show">{t('gitCommands.notes.show')}</option>
                <option value="list">{t('gitCommands.notes.list')}</option>
                <option value="remove">{t('gitCommands.notes.remove')}</option>
                <option value="append">{t('gitCommands.notes.append')}</option>
              </select>
            </div>
            <div>
              <label for="notes-commit" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.notes.commit')}
              </label>
              <input
                id="notes-commit"
                type="text"
                bind:value={notesCommit}
                placeholder="HEAD"
                class="input"
              />
            </div>
            {#if notesOperation === 'add' || notesOperation === 'append'}
              <div>
                <label for="notes-message" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                  {t('gitCommands.notes.message')}
                </label>
                <input
                  id="notes-message"
                  type="text"
                  bind:value={notesMessage}
                  placeholder={t('gitCommands.notes.messagePlaceholder')}
                  class="input"
                />
              </div>
            {/if}
            <div>
              <label for="notes-ref" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.notes.ref')}
              </label>
              <input
                id="notes-ref"
                type="text"
                bind:value={notesRef}
                placeholder="refs/notes/commits"
                class="input"
              />
            </div>
          </div>
        {/if}

        <!-- Bundle 参数 -->
        {#if commandType === 'bundle'}
          <div class="space-y-3">
            <div>
              <label for="bundle-operation" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.bundle.operation')}
              </label>
              <select id="bundle-operation" bind:value={bundleOperation} class="input">
                <option value="create">{t('gitCommands.bundle.create')}</option>
                <option value="list-heads">{t('gitCommands.bundle.listHeads')}</option>
                <option value="verify">{t('gitCommands.bundle.verify')}</option>
                <option value="unbundle">{t('gitCommands.bundle.unbundle')}</option>
              </select>
            </div>
            <div>
              <label for="bundle-file" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.bundle.file')}
              </label>
              <input
                id="bundle-file"
                type="text"
                bind:value={bundleFile}
                placeholder={t('gitCommands.bundle.filePlaceholder')}
                class="input"
              />
            </div>
            {#if bundleOperation === 'create'}
              <div>
                <label for="bundle-branch" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                  {t('gitCommands.bundle.branch')}
                </label>
                <input
                  id="bundle-branch"
                  type="text"
                  bind:value={bundleBranch}
                  placeholder={t('gitCommands.bundle.branchPlaceholder')}
                  class="input"
                />
              </div>
              <label class="flex items-center">
                <input type="checkbox" bind:checked={bundleAll} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.bundle.all')}</span>
              </label>
            {/if}
          </div>
        {/if}

        <!-- Sparse-checkout 参数 -->
        {#if commandType === 'sparse-checkout'}
          <div class="space-y-3">
            <div>
              <label for="sparse-checkout-operation" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.sparseCheckout.operation')}
              </label>
              <select id="sparse-checkout-operation" bind:value={sparseCheckoutOperation} class="input">
                <option value="init">{t('gitCommands.sparseCheckout.init')}</option>
                <option value="set">{t('gitCommands.sparseCheckout.set')}</option>
                <option value="add">{t('gitCommands.sparseCheckout.add')}</option>
                <option value="disable">{t('gitCommands.sparseCheckout.disable')}</option>
                <option value="list">{t('gitCommands.sparseCheckout.list')}</option>
              </select>
            </div>
            {#if sparseCheckoutOperation === 'set' || sparseCheckoutOperation === 'add'}
              <div>
                <label for="sparse-checkout-paths" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                  {t('gitCommands.sparseCheckout.paths')}
                </label>
                <input
                  id="sparse-checkout-paths"
                  type="text"
                  bind:value={sparseCheckoutPaths}
                  placeholder={t('gitCommands.sparseCheckout.pathsPlaceholder')}
                  class="input"
                />
              </div>
            {/if}
            {#if sparseCheckoutOperation === 'init'}
              <label class="flex items-center">
                <input type="checkbox" bind:checked={sparseCheckoutCone} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('gitCommands.sparseCheckout.cone')}</span>
              </label>
            {/if}
          </div>
        {/if}

        <!-- Maintenance 参数 -->
        {#if commandType === 'maintenance'}
          <div class="space-y-3">
            <div>
              <label for="maintenance-operation" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('gitCommands.maintenance.operation')}
              </label>
              <select id="maintenance-operation" bind:value={maintenanceOperation} class="input">
                <option value="start">{t('gitCommands.maintenance.start')}</option>
                <option value="stop">{t('gitCommands.maintenance.stop')}</option>
                <option value="run">{t('gitCommands.maintenance.run')}</option>
              </select>
            </div>
            {#if maintenanceOperation === 'run'}
              <div>
                <label for="maintenance-task" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                  {t('gitCommands.maintenance.task')}
                </label>
                <select id="maintenance-task" bind:value={maintenanceTask} class="input">
                  <option value="gc">{t('gitCommands.maintenance.gc')}</option>
                  <option value="commit-graph">{t('gitCommands.maintenance.commitGraph')}</option>
                  <option value="prefetch">{t('gitCommands.maintenance.prefetch')}</option>
                  <option value="loose-objects">{t('gitCommands.maintenance.looseObjects')}</option>
                  <option value="incremental-repack">{t('gitCommands.maintenance.incrementalRepack')}</option>
                </select>
              </div>
            {/if}
          </div>
        {/if}
      </div>

      <!-- 右侧：生成的命令 -->
      <div>
        <label for="generated-command" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
          {t('gitCommands.generatedCommand')}
        </label>
        <textarea
          id="generated-command"
          readonly
          bind:value={generatedCommand}
          class="textarea font-mono text-sm min-h-[200px]"
        ></textarea>
        {#if commandDescription}
          <div class="mt-3 p-3 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-md">
            <div class="flex items-start gap-2">
              <div class="text-blue-600 dark:text-blue-400 mt-0.5">
                <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                  <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd" />
                </svg>
              </div>
              <div class="flex-1">
                <p class="text-sm font-medium text-blue-900 dark:text-blue-200 mb-1">
                  {t('gitCommands.description')}
                </p>
                <p class="text-sm text-blue-800 dark:text-blue-300 mb-2">
                  {commandDescription}
                </p>
                {#if parameterDescriptions.length > 0}
                  <div class="mt-2 pt-2 border-t border-blue-200 dark:border-blue-700">
                    <p class="text-xs font-medium text-blue-900 dark:text-blue-200 mb-1">
                      {t('gitCommands.parameterDescriptions')}
                    </p>
                    <ul class="text-xs text-blue-800 dark:text-blue-300 space-y-1">
                      {#each parameterDescriptions as desc}
                        <li class="flex items-start gap-1">
                          <span class="text-blue-500 dark:text-blue-400 mt-0.5">•</span>
                          <span>{desc}</span>
                        </li>
                      {/each}
                    </ul>
                  </div>
                {/if}
              </div>
            </div>
          </div>
        {/if}
        <p class="text-xs text-gray-500 dark:text-gray-400 mt-2">
          {t('gitCommands.commandHint')}
        </p>
      </div>
    </div>
  </div>
</div>
