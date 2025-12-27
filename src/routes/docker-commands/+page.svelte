<script lang="ts">
  import { translationsStore } from '$lib/stores/i18n';
  import { Copy, Check, Trash2, Container } from 'lucide-svelte';

  let translations = $derived($translationsStore);

  function t(key: string): string {
    const keys = key.split('.');
    let value: any = translations;
    for (const k of keys) {
      value = value?.[k];
    }
    return value || key;
  }

  type CommandType = 'run' | 'build' | 'ps' | 'images' | 'stop' | 'start' | 'restart' | 'rm' | 'rmi' | 'exec' | 'logs' | 'pull' | 'push' | 'tag' | 'inspect' | 'cp' | 'commit' | 'search' | 'stats' | 'top' | 'port' | 'diff' | 'export' | 'import' | 'save' | 'load' | 'login' | 'logout' | 'network' | 'volume' | 'compose';

  let commandType = $state<CommandType>('run');
  let generatedCommand = $state('');
  let commandDescription = $state('');
  let parameterDescriptions = $state<string[]>([]);
  let copied = $state(false);
  const STORAGE_KEY = 'dockerCommands.state.v1';
  let hasLoadedFromStorage = false;

  // Run 相关
  let runImage = $state('');
  let runName = $state('');
  let runDetached = $state(false);
  let runInteractive = $state(false);
  let runTty = $state(false);
  let runPort = $state('');
  let runVolume = $state('');
  let runEnv = $state('');
  let runCommand = $state('');
  let runRemove = $state(false);
  let runRestart = $state('');

  // Build 相关
  let buildPath = $state('.');
  let buildTag = $state('');
  let buildFile = $state('');
  let buildNoCache = $state(false);
  let buildPull = $state(false);
  let buildTarget = $state('');

  // PS 相关
  let psAll = $state(false);
  let psLatest = $state(false);
  let psFilter = $state('');

  // Images 相关
  let imagesAll = $state(false);
  let imagesFilter = $state('');

  // Stop/Start/Restart 相关
  let containerName = $state('');

  // Rm 相关
  let rmForce = $state(false);
  let rmVolumes = $state(false);

  // Rmi 相关
  let rmiForce = $state(false);
  let rmiNoPrune = $state(false);

  // Exec 相关
  let execInteractive = $state(false);
  let execTty = $state(false);
  let execUser = $state('');
  let execWorkdir = $state('');
  let execCommand = $state('');

  // Logs 相关
  let logsFollow = $state(false);
  let logsTail = $state('');
  let logsSince = $state('');
  let logsUntil = $state('');
  let logsTimestamps = $state(false);

  // Pull/Push 相关
  let imageName = $state('');

  // Tag 相关
  let tagSource = $state('');
  let tagTarget = $state('');

  // Inspect 相关
  let inspectFormat = $state('');
  let inspectType = $state('container');

  // Cp 相关
  let cpSource = $state('');
  let cpDestination = $state('');

  // Commit 相关
  let commitContainer = $state('');
  let commitRepository = $state('');
  let commitTag = $state('');
  let commitMessage = $state('');
  let commitAuthor = $state('');
  let commitPause = $state(true);

  // Search 相关
  let searchTerm = $state('');
  let searchLimit = $state(25);
  let searchStars = $state('');

  // Stats 相关
  let statsAll = $state(false);
  let statsNoStream = $state(false);
  let statsFormat = $state('');

  // Top 相关
  let topArgs = $state('');

  // Port 相关
  let portContainer = $state('');

  // Diff 相关
  let diffContainer = $state('');

  // Export 相关
  let exportContainer = $state('');
  let exportOutput = $state('');

  // Import 相关
  let importFile = $state('');
  let importRepository = $state('');

  // Save 相关
  let saveImage = $state('');
  let saveOutput = $state('');

  // Load 相关
  let loadInput = $state('');

  // Login 相关
  let loginUsername = $state('');
  let loginPassword = $state('');
  let loginServer = $state('');

  // Network 相关
  let networkOperation = $state<'create' | 'ls' | 'inspect' | 'rm' | 'prune' | 'connect' | 'disconnect'>('create');
  let networkName = $state('');
  let networkDriver = $state('bridge');
  let networkSubnet = $state('');
  let networkGateway = $state('');

  // Volume 相关
  let volumeOperation = $state<'create' | 'ls' | 'inspect' | 'rm' | 'prune'>('create');
  let volumeName = $state('');
  let volumeDriver = $state('local');

  // Compose 相关
  let composeOperation = $state<'up' | 'down' | 'build' | 'start' | 'stop' | 'restart' | 'ps' | 'logs' | 'exec'>('up');
  let composeFile = $state('docker-compose.yml');
  let composeDetached = $state(false);
  let composeBuild = $state(false);
  let composeService = $state('');

  // 从本地存储恢复状态
  function loadSavedState() {
    if (hasLoadedFromStorage) return;
    hasLoadedFromStorage = true;
    try {
      const saved = localStorage.getItem(STORAGE_KEY);
      if (saved) {
        const parsed = JSON.parse(saved);
        commandType = parsed.commandType || 'run';
        runImage = parsed.runImage || '';
        buildPath = parsed.buildPath || '.';
        containerName = parsed.containerName || '';
        imageName = parsed.imageName || '';
      }
    } catch (error) {
      console.error('Failed to load Docker commands state:', error);
    }
  }

  // 保存状态
  function saveState() {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify({
        commandType,
        runImage,
        buildPath,
        containerName,
        imageName
      }));
    } catch (error) {
      console.error('Failed to save Docker commands state:', error);
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
      case 'run':
        cmd = 'docker run';
        if (runDetached) {
          cmd += ' -d';
        }
        if (runInteractive) {
          cmd += ' -i';
        }
        if (runTty) {
          cmd += ' -t';
        }
        if (runName) {
          cmd += ` --name ${runName}`;
        }
        if (runPort) {
          const ports = runPort.split(',').map(p => p.trim()).filter(p => p);
          ports.forEach(port => {
            cmd += ` -p ${port}`;
          });
        }
        if (runVolume) {
          const volumes = runVolume.split(',').map(v => v.trim()).filter(v => v);
          volumes.forEach(vol => {
            cmd += ` -v ${vol}`;
          });
        }
        if (runEnv) {
          const envs = runEnv.split(',').map(e => e.trim()).filter(e => e);
          envs.forEach(env => {
            cmd += ` -e ${env}`;
          });
        }
        if (runRestart) {
          cmd += ` --restart ${runRestart}`;
        }
        if (runRemove) {
          cmd += ' --rm';
        }
        cmd += ` ${runImage || '<image>'}`;
        if (runCommand) {
          cmd += ` ${runCommand}`;
        }
        break;

      case 'build':
        cmd = 'docker build';
        if (buildTag) {
          cmd += ` -t ${buildTag}`;
        }
        if (buildFile) {
          cmd += ` -f ${buildFile}`;
        }
        if (buildNoCache) {
          cmd += ' --no-cache';
        }
        if (buildPull) {
          cmd += ' --pull';
        }
        if (buildTarget) {
          cmd += ` --target ${buildTarget}`;
        }
        cmd += ` ${buildPath || '.'}`;
        break;

      case 'ps':
        cmd = 'docker ps';
        if (psAll) {
          cmd += ' -a';
        }
        if (psLatest) {
          cmd += ' -l';
        }
        if (psFilter) {
          cmd += ` --filter "${psFilter}"`;
        }
        break;

      case 'images':
        cmd = 'docker images';
        if (imagesAll) {
          cmd += ' -a';
        }
        if (imagesFilter) {
          cmd += ` --filter "${imagesFilter}"`;
        }
        break;

      case 'stop':
        cmd = `docker stop ${containerName || '<container>'}`;
        break;

      case 'start':
        cmd = `docker start ${containerName || '<container>'}`;
        break;

      case 'restart':
        cmd = `docker restart ${containerName || '<container>'}`;
        break;

      case 'rm':
        cmd = 'docker rm';
        if (rmForce) {
          cmd += ' -f';
        }
        if (rmVolumes) {
          cmd += ' -v';
        }
        cmd += ` ${containerName || '<container>'}`;
        break;

      case 'rmi':
        cmd = 'docker rmi';
        if (rmiForce) {
          cmd += ' -f';
        }
        if (rmiNoPrune) {
          cmd += ' --no-prune';
        }
        cmd += ` ${imageName || '<image>'}`;
        break;

      case 'exec':
        cmd = 'docker exec';
        if (execInteractive) {
          cmd += ' -i';
        }
        if (execTty) {
          cmd += ' -t';
        }
        if (execUser) {
          cmd += ` -u ${execUser}`;
        }
        if (execWorkdir) {
          cmd += ` -w ${execWorkdir}`;
        }
        cmd += ` ${containerName || '<container>'}`;
        cmd += ` ${execCommand || '<command>'}`;
        break;

      case 'logs':
        cmd = 'docker logs';
        if (logsFollow) {
          cmd += ' -f';
        }
        if (logsTail) {
          cmd += ` --tail ${logsTail}`;
        }
        if (logsSince) {
          cmd += ` --since ${logsSince}`;
        }
        if (logsUntil) {
          cmd += ` --until ${logsUntil}`;
        }
        if (logsTimestamps) {
          cmd += ' -t';
        }
        cmd += ` ${containerName || '<container>'}`;
        break;

      case 'pull':
        cmd = `docker pull ${imageName || '<image>'}`;
        break;

      case 'push':
        cmd = `docker push ${imageName || '<image>'}`;
        break;

      case 'tag':
        cmd = `docker tag ${tagSource || '<source-image>'} ${tagTarget || '<target-image>'}`;
        break;

      case 'inspect':
        cmd = 'docker inspect';
        if (inspectFormat) {
          cmd += ` --format "${inspectFormat}"`;
        }
        if (inspectType === 'image') {
          cmd += ` --type image`;
        }
        cmd += ` ${containerName || imageName || '<name>'}`;
        break;

      case 'cp':
        cmd = `docker cp ${cpSource || '<source>'} ${cpDestination || '<destination>'}`;
        break;

      case 'commit':
        cmd = 'docker commit';
        if (commitPause) {
          cmd += ' -p';
        }
        if (commitAuthor) {
          cmd += ` --author "${commitAuthor}"`;
        }
        if (commitMessage) {
          cmd += ` -m "${commitMessage}"`;
        }
        cmd += ` ${commitContainer || '<container>'}`;
        if (commitRepository) {
          cmd += ` ${commitRepository}`;
          if (commitTag) {
            cmd += `:${commitTag}`;
          }
        }
        break;

      case 'search':
        cmd = 'docker search';
        if (searchLimit) {
          cmd += ` --limit ${searchLimit}`;
        }
        if (searchStars) {
          cmd += ` --stars ${searchStars}`;
        }
        cmd += ` ${searchTerm || '<term>'}`;
        break;

      case 'stats':
        cmd = 'docker stats';
        if (statsAll) {
          cmd += ' -a';
        }
        if (statsNoStream) {
          cmd += ' --no-stream';
        }
        if (statsFormat) {
          cmd += ` --format "${statsFormat}"`;
        }
        if (containerName) {
          cmd += ` ${containerName}`;
        }
        break;

      case 'top':
        cmd = `docker top ${containerName || '<container>'}`;
        if (topArgs) {
          cmd += ` ${topArgs}`;
        }
        break;

      case 'port':
        cmd = `docker port ${portContainer || '<container>'}`;
        break;

      case 'diff':
        cmd = `docker diff ${diffContainer || '<container>'}`;
        break;

      case 'export':
        cmd = `docker export ${exportContainer || '<container>'}`;
        if (exportOutput) {
          cmd += ` -o ${exportOutput}`;
        } else {
          cmd += ' > <output-file>.tar';
        }
        break;

      case 'import':
        cmd = 'docker import';
        cmd += ` ${importFile || '<file>'}`;
        cmd += ` ${importRepository || '<repository>'}`;
        break;

      case 'save':
        cmd = `docker save ${saveImage || '<image>'}`;
        if (saveOutput) {
          cmd += ` -o ${saveOutput}`;
        } else {
          cmd += ' > <output-file>.tar';
        }
        break;

      case 'load':
        cmd = `docker load -i ${loadInput || '<input-file>'}`;
        break;

      case 'login':
        cmd = 'docker login';
        if (loginUsername) {
          cmd += ` -u ${loginUsername}`;
        }
        if (loginPassword) {
          cmd += ` -p ${loginPassword}`;
        }
        if (loginServer) {
          cmd += ` ${loginServer}`;
        }
        break;

      case 'logout':
        cmd = 'docker logout';
        if (loginServer) {
          cmd += ` ${loginServer}`;
        }
        break;

      case 'network':
        if (networkOperation === 'create') {
          cmd = 'docker network create';
          if (networkDriver !== 'bridge') {
            cmd += ` --driver ${networkDriver}`;
          }
          if (networkSubnet) {
            cmd += ` --subnet ${networkSubnet}`;
          }
          if (networkGateway) {
            cmd += ` --gateway ${networkGateway}`;
          }
          cmd += ` ${networkName || '<network-name>'}`;
        } else if (networkOperation === 'ls') {
          cmd = 'docker network ls';
        } else if (networkOperation === 'inspect') {
          cmd = `docker network inspect ${networkName || '<network>'}`;
        } else if (networkOperation === 'rm') {
          cmd = `docker network rm ${networkName || '<network>'}`;
        } else if (networkOperation === 'prune') {
          cmd = 'docker network prune';
        } else if (networkOperation === 'connect') {
          cmd = `docker network connect ${networkName || '<network>'} ${containerName || '<container>'}`;
        } else if (networkOperation === 'disconnect') {
          cmd = `docker network disconnect ${networkName || '<network>'} ${containerName || '<container>'}`;
        }
        break;

      case 'volume':
        if (volumeOperation === 'create') {
          cmd = 'docker volume create';
          if (volumeDriver !== 'local') {
            cmd += ` --driver ${volumeDriver}`;
          }
          cmd += ` ${volumeName || '<volume-name>'}`;
        } else if (volumeOperation === 'ls') {
          cmd = 'docker volume ls';
        } else if (volumeOperation === 'inspect') {
          cmd = `docker volume inspect ${volumeName || '<volume>'}`;
        } else if (volumeOperation === 'rm') {
          cmd = `docker volume rm ${volumeName || '<volume>'}`;
        } else if (volumeOperation === 'prune') {
          cmd = 'docker volume prune';
        }
        break;

      case 'compose':
        if (composeOperation === 'up') {
          cmd = 'docker-compose up';
          if (composeDetached) {
            cmd += ' -d';
          }
          if (composeBuild) {
            cmd += ' --build';
          }
          if (composeFile !== 'docker-compose.yml') {
            cmd += ` -f ${composeFile}`;
          }
          if (composeService) {
            cmd += ` ${composeService}`;
          }
        } else if (composeOperation === 'down') {
          cmd = 'docker-compose down';
          if (composeFile !== 'docker-compose.yml') {
            cmd += ` -f ${composeFile}`;
          }
        } else if (composeOperation === 'build') {
          cmd = 'docker-compose build';
          if (composeFile !== 'docker-compose.yml') {
            cmd += ` -f ${composeFile}`;
          }
          if (composeService) {
            cmd += ` ${composeService}`;
          }
        } else if (composeOperation === 'start') {
          cmd = 'docker-compose start';
          if (composeFile !== 'docker-compose.yml') {
            cmd += ` -f ${composeFile}`;
          }
          if (composeService) {
            cmd += ` ${composeService}`;
          }
        } else if (composeOperation === 'stop') {
          cmd = 'docker-compose stop';
          if (composeFile !== 'docker-compose.yml') {
            cmd += ` -f ${composeFile}`;
          }
          if (composeService) {
            cmd += ` ${composeService}`;
          }
        } else if (composeOperation === 'restart') {
          cmd = 'docker-compose restart';
          if (composeFile !== 'docker-compose.yml') {
            cmd += ` -f ${composeFile}`;
          }
          if (composeService) {
            cmd += ` ${composeService}`;
          }
        } else if (composeOperation === 'ps') {
          cmd = 'docker-compose ps';
          if (composeFile !== 'docker-compose.yml') {
            cmd += ` -f ${composeFile}`;
          }
        } else if (composeOperation === 'logs') {
          cmd = 'docker-compose logs';
          if (composeFile !== 'docker-compose.yml') {
            cmd += ` -f ${composeFile}`;
          }
          if (composeService) {
            cmd += ` ${composeService}`;
          }
        } else if (composeOperation === 'exec') {
          cmd = 'docker-compose exec';
          if (composeFile !== 'docker-compose.yml') {
            cmd += ` -f ${composeFile}`;
          }
          cmd += ` ${composeService || '<service>'}`;
          cmd += ` <command>`;
        }
        break;
    }

    generatedCommand = cmd;
    commandDescription = getCommandDescription(commandType);
    parameterDescriptions = getParameterDescriptions(commandType);
  }

  function getCommandDescription(type: CommandType): string {
    return t(`dockerCommands.descriptions.${type}`);
  }

  function getParameterDescriptions(type: CommandType): string[] {
    const descriptions: string[] = [];
    
    switch (type) {
      case 'run':
        if (runDetached) descriptions.push(t('dockerCommands.paramDescriptions.run.detached'));
        if (runInteractive) descriptions.push(t('dockerCommands.paramDescriptions.run.interactive'));
        if (runTty) descriptions.push(t('dockerCommands.paramDescriptions.run.tty'));
        if (runName) descriptions.push(t('dockerCommands.paramDescriptions.run.name'));
        if (runPort) descriptions.push(t('dockerCommands.paramDescriptions.run.port'));
        if (runVolume) descriptions.push(t('dockerCommands.paramDescriptions.run.volume'));
        if (runEnv) descriptions.push(t('dockerCommands.paramDescriptions.run.env'));
        if (runRestart) descriptions.push(t('dockerCommands.paramDescriptions.run.restart'));
        if (runRemove) descriptions.push(t('dockerCommands.paramDescriptions.run.remove'));
        break;
      case 'build':
        if (buildTag) descriptions.push(t('dockerCommands.paramDescriptions.build.tag'));
        if (buildFile) descriptions.push(t('dockerCommands.paramDescriptions.build.file'));
        if (buildNoCache) descriptions.push(t('dockerCommands.paramDescriptions.build.noCache'));
        if (buildPull) descriptions.push(t('dockerCommands.paramDescriptions.build.pull'));
        if (buildTarget) descriptions.push(t('dockerCommands.paramDescriptions.build.target'));
        break;
      case 'ps':
        if (psAll) descriptions.push(t('dockerCommands.paramDescriptions.ps.all'));
        if (psLatest) descriptions.push(t('dockerCommands.paramDescriptions.ps.latest'));
        if (psFilter) descriptions.push(t('dockerCommands.paramDescriptions.ps.filter'));
        break;
      case 'images':
        if (imagesAll) descriptions.push(t('dockerCommands.paramDescriptions.images.all'));
        if (imagesFilter) descriptions.push(t('dockerCommands.paramDescriptions.images.filter'));
        break;
      case 'rm':
        if (rmForce) descriptions.push(t('dockerCommands.paramDescriptions.rm.force'));
        if (rmVolumes) descriptions.push(t('dockerCommands.paramDescriptions.rm.volumes'));
        break;
      case 'rmi':
        if (rmiForce) descriptions.push(t('dockerCommands.paramDescriptions.rmi.force'));
        if (rmiNoPrune) descriptions.push(t('dockerCommands.paramDescriptions.rmi.noPrune'));
        break;
      case 'exec':
        if (execInteractive) descriptions.push(t('dockerCommands.paramDescriptions.exec.interactive'));
        if (execTty) descriptions.push(t('dockerCommands.paramDescriptions.exec.tty'));
        if (execUser) descriptions.push(t('dockerCommands.paramDescriptions.exec.user'));
        if (execWorkdir) descriptions.push(t('dockerCommands.paramDescriptions.exec.workdir'));
        break;
      case 'logs':
        if (logsFollow) descriptions.push(t('dockerCommands.paramDescriptions.logs.follow'));
        if (logsTail) descriptions.push(t('dockerCommands.paramDescriptions.logs.tail'));
        if (logsSince) descriptions.push(t('dockerCommands.paramDescriptions.logs.since'));
        if (logsUntil) descriptions.push(t('dockerCommands.paramDescriptions.logs.until'));
        if (logsTimestamps) descriptions.push(t('dockerCommands.paramDescriptions.logs.timestamps'));
        break;
      case 'commit':
        if (commitPause) descriptions.push(t('dockerCommands.paramDescriptions.commit.pause'));
        if (commitAuthor) descriptions.push(t('dockerCommands.paramDescriptions.commit.author'));
        if (commitMessage) descriptions.push(t('dockerCommands.paramDescriptions.commit.message'));
        break;
      case 'search':
        if (searchLimit) descriptions.push(t('dockerCommands.paramDescriptions.search.limit'));
        if (searchStars) descriptions.push(t('dockerCommands.paramDescriptions.search.stars'));
        break;
      case 'stats':
        if (statsAll) descriptions.push(t('dockerCommands.paramDescriptions.stats.all'));
        if (statsNoStream) descriptions.push(t('dockerCommands.paramDescriptions.stats.noStream'));
        if (statsFormat) descriptions.push(t('dockerCommands.paramDescriptions.stats.format'));
        break;
      case 'network':
        if (networkDriver !== 'bridge') descriptions.push(t('dockerCommands.paramDescriptions.network.driver'));
        if (networkSubnet) descriptions.push(t('dockerCommands.paramDescriptions.network.subnet'));
        if (networkGateway) descriptions.push(t('dockerCommands.paramDescriptions.network.gateway'));
        break;
      case 'volume':
        if (volumeDriver !== 'local') descriptions.push(t('dockerCommands.paramDescriptions.volume.driver'));
        break;
      case 'compose':
        if (composeDetached) descriptions.push(t('dockerCommands.paramDescriptions.compose.detached'));
        if (composeBuild) descriptions.push(t('dockerCommands.paramDescriptions.compose.build'));
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
    runImage = '';
    runName = '';
    runPort = '';
    runVolume = '';
    runEnv = '';
    runCommand = '';
    buildPath = '.';
    buildTag = '';
    buildFile = '';
    containerName = '';
    imageName = '';
    tagSource = '';
    tagTarget = '';
    commitContainer = '';
    commitRepository = '';
    commitTag = '';
    commitMessage = '';
    searchTerm = '';
    exportContainer = '';
    exportOutput = '';
    importFile = '';
    importRepository = '';
    saveImage = '';
    saveOutput = '';
    loadInput = '';
    loginUsername = '';
    loginPassword = '';
    loginServer = '';
    networkName = '';
    volumeName = '';
    composeFile = 'docker-compose.yml';
    composeService = '';
    generatedCommand = '';
  }

  // 相关字段变化时自动生成
  $effect(() => {
    runImage;
    runName;
    runDetached;
    runInteractive;
    runTty;
    runPort;
    runVolume;
    runEnv;
    runCommand;
    runRemove;
    runRestart;
    buildPath;
    buildTag;
    buildFile;
    buildNoCache;
    buildPull;
    buildTarget;
    psAll;
    psLatest;
    psFilter;
    imagesAll;
    imagesFilter;
    containerName;
    rmForce;
    rmVolumes;
    rmiForce;
    rmiNoPrune;
    execInteractive;
    execTty;
    execUser;
    execWorkdir;
    execCommand;
    logsFollow;
    logsTail;
    logsSince;
    logsUntil;
    logsTimestamps;
    imageName;
    tagSource;
    tagTarget;
    inspectFormat;
    inspectType;
    cpSource;
    cpDestination;
    commitPause;
    commitAuthor;
    commitMessage;
    searchTerm;
    searchLimit;
    searchStars;
    statsAll;
    statsNoStream;
    statsFormat;
    topArgs;
    portContainer;
    diffContainer;
    exportOutput;
    importFile;
    importRepository;
    saveOutput;
    loadInput;
    loginUsername;
    loginPassword;
    loginServer;
    networkOperation;
    networkName;
    networkDriver;
    networkSubnet;
    networkGateway;
    volumeOperation;
    volumeName;
    volumeDriver;
    composeOperation;
    composeFile;
    composeDetached;
    composeBuild;
    composeService;
    generateCommand();
    parameterDescriptions = getParameterDescriptions(commandType);
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
            {t('dockerCommands.clear')}
          </button>
        </div>
      </div>
    {/if}

    <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
      <!-- 左侧：命令类型和参数 -->
      <div class="space-y-4">
        <div>
          <label for="command-type" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
            {t('dockerCommands.commandType')}
          </label>
          <select id="command-type" bind:value={commandType} class="input">
            <option value="run">{t('dockerCommands.types.run')}</option>
            <option value="build">{t('dockerCommands.types.build')}</option>
            <option value="ps">{t('dockerCommands.types.ps')}</option>
            <option value="images">{t('dockerCommands.types.images')}</option>
            <option value="stop">{t('dockerCommands.types.stop')}</option>
            <option value="start">{t('dockerCommands.types.start')}</option>
            <option value="restart">{t('dockerCommands.types.restart')}</option>
            <option value="rm">{t('dockerCommands.types.rm')}</option>
            <option value="rmi">{t('dockerCommands.types.rmi')}</option>
            <option value="exec">{t('dockerCommands.types.exec')}</option>
            <option value="logs">{t('dockerCommands.types.logs')}</option>
            <option value="pull">{t('dockerCommands.types.pull')}</option>
            <option value="push">{t('dockerCommands.types.push')}</option>
            <option value="tag">{t('dockerCommands.types.tag')}</option>
            <option value="inspect">{t('dockerCommands.types.inspect')}</option>
            <option value="cp">{t('dockerCommands.types.cp')}</option>
            <option value="commit">{t('dockerCommands.types.commit')}</option>
            <option value="search">{t('dockerCommands.types.search')}</option>
            <option value="stats">{t('dockerCommands.types.stats')}</option>
            <option value="top">{t('dockerCommands.types.top')}</option>
            <option value="port">{t('dockerCommands.types.port')}</option>
            <option value="diff">{t('dockerCommands.types.diff')}</option>
            <option value="export">{t('dockerCommands.types.export')}</option>
            <option value="import">{t('dockerCommands.types.import')}</option>
            <option value="save">{t('dockerCommands.types.save')}</option>
            <option value="load">{t('dockerCommands.types.load')}</option>
            <option value="login">{t('dockerCommands.types.login')}</option>
            <option value="logout">{t('dockerCommands.types.logout')}</option>
            <option value="network">{t('dockerCommands.types.network')}</option>
            <option value="volume">{t('dockerCommands.types.volume')}</option>
            <option value="compose">{t('dockerCommands.types.compose')}</option>
          </select>
        </div>

        <!-- Run 参数 -->
        {#if commandType === 'run'}
          <div class="space-y-3">
            <div>
              <label for="run-image" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('dockerCommands.run.image')}
              </label>
              <input
                id="run-image"
                type="text"
                bind:value={runImage}
                placeholder={t('dockerCommands.run.imagePlaceholder')}
                class="input"
              />
            </div>
            <div>
              <label for="run-name" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('dockerCommands.run.name')}
              </label>
              <input
                id="run-name"
                type="text"
                bind:value={runName}
                placeholder={t('dockerCommands.run.namePlaceholder')}
                class="input"
              />
            </div>
            <div>
              <label for="run-port" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('dockerCommands.run.port')}
              </label>
              <input
                id="run-port"
                type="text"
                bind:value={runPort}
                placeholder={t('dockerCommands.run.portPlaceholder')}
                class="input"
              />
            </div>
            <div>
              <label for="run-volume" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('dockerCommands.run.volume')}
              </label>
              <input
                id="run-volume"
                type="text"
                bind:value={runVolume}
                placeholder={t('dockerCommands.run.volumePlaceholder')}
                class="input"
              />
            </div>
            <div>
              <label for="run-env" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('dockerCommands.run.env')}
              </label>
              <input
                id="run-env"
                type="text"
                bind:value={runEnv}
                placeholder={t('dockerCommands.run.envPlaceholder')}
                class="input"
              />
            </div>
            <div>
              <label for="run-command" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('dockerCommands.run.command')}
              </label>
              <input
                id="run-command"
                type="text"
                bind:value={runCommand}
                placeholder={t('dockerCommands.run.commandPlaceholder')}
                class="input"
              />
            </div>
            <div>
              <label for="run-restart" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('dockerCommands.run.restart')}
              </label>
              <select id="run-restart" bind:value={runRestart} class="input">
                <option value="">{t('dockerCommands.run.restartNone')}</option>
                <option value="no">{t('dockerCommands.run.restartNo')}</option>
                <option value="always">{t('dockerCommands.run.restartAlways')}</option>
                <option value="on-failure">{t('dockerCommands.run.restartOnFailure')}</option>
                <option value="unless-stopped">{t('dockerCommands.run.restartUnlessStopped')}</option>
              </select>
            </div>
            <div class="flex items-center gap-4 flex-wrap">
              <label class="flex items-center">
                <input type="checkbox" bind:checked={runDetached} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('dockerCommands.run.detached')}</span>
              </label>
              <label class="flex items-center">
                <input type="checkbox" bind:checked={runInteractive} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('dockerCommands.run.interactive')}</span>
              </label>
              <label class="flex items-center">
                <input type="checkbox" bind:checked={runTty} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('dockerCommands.run.tty')}</span>
              </label>
              <label class="flex items-center">
                <input type="checkbox" bind:checked={runRemove} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('dockerCommands.run.remove')}</span>
              </label>
            </div>
          </div>
        {/if}

        <!-- Build 参数 -->
        {#if commandType === 'build'}
          <div class="space-y-3">
            <div>
              <label for="build-path" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('dockerCommands.build.path')}
              </label>
              <input
                id="build-path"
                type="text"
                bind:value={buildPath}
                placeholder="."
                class="input"
              />
            </div>
            <div>
              <label for="build-tag" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('dockerCommands.build.tag')}
              </label>
              <input
                id="build-tag"
                type="text"
                bind:value={buildTag}
                placeholder={t('dockerCommands.build.tagPlaceholder')}
                class="input"
              />
            </div>
            <div>
              <label for="build-file" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('dockerCommands.build.file')}
              </label>
              <input
                id="build-file"
                type="text"
                bind:value={buildFile}
                placeholder="Dockerfile"
                class="input"
              />
            </div>
            <div>
              <label for="build-target" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
                {t('dockerCommands.build.target')}
              </label>
              <input
                id="build-target"
                type="text"
                bind:value={buildTarget}
                placeholder={t('dockerCommands.build.targetPlaceholder')}
                class="input"
              />
            </div>
            <div class="flex items-center gap-4">
              <label class="flex items-center">
                <input type="checkbox" bind:checked={buildNoCache} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('dockerCommands.build.noCache')}</span>
              </label>
              <label class="flex items-center">
                <input type="checkbox" bind:checked={buildPull} class="mr-2" />
                <span class="text-sm text-gray-700 dark:text-gray-300">{t('dockerCommands.build.pull')}</span>
              </label>
            </div>
          </div>
        {/if}

        <!-- 其他命令类型的参数 UI 可以继续添加... -->
        <!-- 为了节省空间，这里只展示主要命令的参数 -->
        <!-- 其他命令的参数可以按照类似模式添加 -->

      </div>

      <!-- 右侧：生成的命令 -->
      <div>
        <label for="generated-command" class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">
          {t('dockerCommands.generatedCommand')}
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
                  {t('dockerCommands.description')}
                </p>
                <p class="text-sm text-blue-800 dark:text-blue-300 mb-2">
                  {commandDescription}
                </p>
                {#if parameterDescriptions.length > 0}
                  <div class="mt-2 pt-2 border-t border-blue-200 dark:border-blue-700">
                    <p class="text-xs font-medium text-blue-900 dark:text-blue-200 mb-1">
                      {t('dockerCommands.parameterDescriptions')}
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
          {t('dockerCommands.commandHint')}
        </p>
      </div>
    </div>
  </div>
</div>
