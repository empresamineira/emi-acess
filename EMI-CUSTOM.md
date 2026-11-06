# EMI Acess: personalização do RustDesk

Cliente de acesso remoto da Empresa Mineira de Informática, derivado do RustDesk 1.5.1 (AGPL-3.0). Este arquivo explica o que foi alterado, como gerar o instalador, como trocar servidor, chave e senha, e como atualizar a partir do upstream.

- **Fork:** `empresamineira/rustdesk`, branch `emi-custom`
- **Upstream:** `rustdesk/rustdesk`, remote `upstream`
- **Licença:** AGPL-3.0 (`LICENCE`). O copyright original e os créditos foram mantidos. A tela Sobre aponta para o código-fonte deste fork, que precisa continuar disponível para quem usa o cliente.

## Como gerar o instalador

### Configuração inicial no GitHub (uma vez)

1. Ative o GitHub Actions no fork, na aba **Actions**.
2. Em **Settings > Secrets and variables > Actions**, crie os secrets da tabela abaixo.
3. Em **Settings > General > Default branch**, defina `emi-custom`. O GitHub só mostra o botão de execução manual para workflows que existem no branch padrão.
4. Na aba **Actions**, desative `flutter-nightly`, `update-webpki-roots`, `ci` e `flutter-ci`. Sem isso, o agendamento diário do `flutter-nightly` passa a rodar no `emi-custom` e compila todas as plataformas.

| Secret | Conteúdo | Se faltar |
|---|---|---|
| `RENDEZVOUS_SERVER` | Endereço do servidor ID, por exemplo `hbbs.exemplo.com.br` | O workflow falha, para não gerar cliente apontando para o servidor público |
| `RS_PUB_KEY` | Chave pública do servidor (conteúdo do `id_ed25519.pub`) | O workflow falha |
| `EMI_SETTINGS_HASH` | SHA-256 da senha, em hex minúsculo | O workflow avisa e o build segue, mas a seção de servidor fica sempre bloqueada |

### Executar

Em **Actions > EMI Acess Windows x64 > Run workflow**, escolha o branch `emi-custom`. Ao terminar, baixe o artifact `EMI-Acess-<versão>-x86_64-install`.

A opção `run-tests` roda os testes do EMI antes de empacotar. Ela deixa o build mais lento, mas mantenha ligada depois de qualquer alteração de código.

O instalador **não tem assinatura de código**, então o Windows SmartScreen e antivírus podem alertar. Para eliminar isso, é preciso um certificado de assinatura.

## Como trocar servidor, chave ou senha

Os três valores são gravados no binário durante a compilação. Depois de trocar um secret, gere e distribua um novo instalador.

**Servidor ou chave:** atualize `RENDEZVOUS_SERVER` e `RS_PUB_KEY`, e rode o workflow. Eles são apenas valores padrão: uma máquina que já tenha um servidor salvo nas configurações continua usando o salvo. Nesse caso, corrija pela tela de configurações ou limpe os campos, para o padrão voltar a valer.

**Senha:** gere o hash sem deixar a senha no histórico do terminal (Git Bash):

```bash
read -rs SENHA; printf '%s' "$SENHA" | sha256sum; unset SENHA
```

Copie só os 64 caracteres para o secret `EMI_SETTINGS_HASH` e rode o workflow. Use uma senha **longa e aleatória** (veja os limites conhecidos).

**Nome ou publicador:** o nome exibido aparece em `src/emi.rs` (`DISPLAY_NAME`, `PUBLISHER`), `flutter/lib/consts.dart` (`kEmiDisplayName`), `flutter/windows/runner/main.cpp`, `flutter/windows/runner/service_control.h`, `flutter/windows/runner/Runner.rc` e `libs/portable/Cargo.toml`. Os valores de `main.cpp`, `src/emi.rs` e `consts.dart` precisam ser iguais, ou segunda instância e links `rustdesk://` param de funcionar. Use só ASCII, sem `" % | < >`.

## O que foi alterado

O nome interno `APP_NAME` continua `RustDesk`, porque ele define serviço, pipes, pasta de configuração, caminho de instalação e registro. Por isso existe um nome de exibição separado. Crates, binários, nome do serviço e caminhos de configuração não mudaram.

### Marca

| Arquivo | Alteração |
|---|---|
| `src/emi.rs` (novo) | Constantes da marca e toda a lógica específica da EMI |
| `src/lib.rs` | Registra o módulo |
| `src/lang.rs` | Troca `RustDesk` pelo nome de exibição em todas as traduções |
| `src/flutter.rs`, `src/core_main.rs`, `flutter/lib/common.dart`, `flutter/windows/runner/main.cpp` | Título da janela e localização da janela existente |
| `flutter/lib/main.dart`, `flutter/lib/consts.dart`, `flutter/lib/desktop/widgets/tabbar_widget.dart` | Título do app e constantes Dart |
| `src/tray.rs` | Texto da bandeja |
| `src/core_main.rs` | Título das notificações de instalação |
| `src/platform/windows.rs` | Atalhos, pasta do Menu Iniciar, Programas e Recursos, nome de exibição do serviço, limpeza dos atalhos do RustDesk oficial |
| `src/auth_2fa.rs` | Emissor do 2FA (afeta só novos cadastros) |
| `src/whiteboard/windows.rs` | Título da janela do quadro |
| `flutter/windows/runner/service_control.h` | Nome de exibição do serviço e mensagem de erro |
| `flutter/windows/runner/Runner.rc`, `libs/portable/Cargo.toml` | Metadados dos executáveis |
| `flutter/lib/desktop/pages/desktop_setting_page.dart` | Crédito AGPL e link do código na tela Sobre |

### Imagens

Geradas a partir de `C:\emi-branding\logo.png` (699×699). Os ICOs têm 16, 24, 32, 48, 64, 128 e 256 px, exceto o da bandeja, que tem 32 px.

| Arquivo | Uso |
|---|---|
| `flutter/windows/runner/resources/app_icon.ico` | Ícone do `rustdesk.exe` |
| `res/icon.ico` | Ícone do instalador |
| `res/tray-icon.ico` | Bandeja, se `icon.png` não existir |
| `flutter/assets/icon.ico` (novo) | Janela |
| `flutter/assets/icon.png` (novo) | Telas e bandeja |
| `flutter/assets/logo.png` (novo) | Logo da tela inicial |

O `.gitignore` tem a regra `*png`. Se trocar por um arquivo novo, use `git add -f`.

Só o Windows foi coberto. Ícones de outras plataformas continuam originais.

### Atualizações bloqueadas

Com `APP_NAME` igual a `RustDesk`, o cliente consultaria as versões oficiais, que substituiriam o build da EMI. `do_check_software_update` (`src/common.rs`) retorna sem consultar nada, e as opções de atualização ficam ocultas em `desktop_setting_page.dart`. Novas versões são distribuídas por vocês, com um novo instalador.

### Servidor pré-configurado

Antes havia `option_env!` em `libs/hbb_common/src/config.rs`, mas o upstream o removeu no commit `33d90c7` do hbb_common, deixando fixos o servidor e a chave públicos. Agora `apply_build_defaults()` (`src/emi.rs`) lê os secrets e preenche `DEFAULT_SETTINGS`. Ela é chamada no início de `load_custom_client()` (`src/common.rs`), que roda na interface e no serviço. O submódulo não foi alterado.

O endereço da API e o do relay são derivados do servidor ID.

### Trava das configurações de rede

O clique em **ID/Relay Server** pede a senha antes de abrir o diálogo, então os valores e os botões de importar e exportar ficam protegidos. A senha é pedida a cada abertura.

| Arquivo | Alteração |
|---|---|
| `src/emi.rs` | Verificação do SHA-256 e limite de tentativas |
| `src/flutter_ffi.rs` | Chave `emi-verify-server-settings:` em `main_get_common_sync`, a ponte já existente. Não precisa rodar codegen |
| `flutter/lib/desktop/widgets/emi_server_gate.dart` (novo) | Diálogo de senha |
| `flutter/lib/desktop/pages/desktop_setting_page.dart` | Passa o clique pelo diálogo |
| `flutter/test/emi_server_gate_test.dart` (novo) | Testes |

- O hash é comparado no Rust e não chega ao Flutter.
- Sem hash válido no build, a seção fica sempre bloqueada.
- Cinco senhas erradas bloqueiam a verificação por 5 minutos. Só a contagem e o horário são gravados, na opção `emi-server-settings-lockout` do `LocalConfig`. A senha nunca é gravada.
- Em instalações, a trava de administrador do RustDesk continua valendo: primeiro o UAC, depois a senha.

### Workflow

`.github/workflows/emi-windows-x64.yml` (novo) gera só o instalador Windows x64. É uma cópia reduzida do job `build-for-windows-flutter` de `flutter-build.yml`, que não foi alterado, pois não tem seleção de alvos. O arquivo termina em `install.exe` para abrir direto a tela de instalação.

## Como atualizar a partir do upstream

Use uma tag estável, não o `master`.

```bash
git fetch upstream --tags
git checkout emi-custom
git checkout -b atualiza-<versão>
git merge <versão>
git submodule update --init --recursive
```

Depois do merge, confira:

1. **Conflitos prováveis:** `src/platform/windows.rs`, `src/lang.rs`, `flutter/windows/runner/Runner.rc`, `libs/portable/Cargo.toml` e os ícones. Em ícones, mantenha os da EMI: `git checkout --ours -- <arquivo>`.
2. **Workflow:** veja o que mudou no job original e leve para `emi-windows-x64.yml`, principalmente `VERSION`, `FLUTTER_VERSION`, `VCPKG_COMMIT_ID` e actions fixadas por hash:
   ```bash
   git diff <versão-antiga> <versão-nova> -- .github/workflows/flutter-build.yml
   ```
3. **Atalhos novos:** procure `.lnk` e `get_app_name` em `src/platform/windows.rs`. Atalho novo deve usar `display_name`. Serviço, processo, registro e caminhos devem continuar com `app_name`.
4. **Textos novos:** procure literais `RustDesk` novos em `flutter/lib/desktop` e `src`.
5. **Verificação de atualização e servidor:** confirme que `do_check_software_update` ainda começa com o retorno antecipado, que `load_custom_client` ainda chama `apply_build_defaults`, e que ninguém passou a ler servidor por outro caminho.
6. **Trava:** procure novas chamadas a `showServerSettings` em `flutter/lib/desktop`. Toda chamada no desktop precisa passar por `showEmiServerGate`.
7. **Teste:** rode o workflow com `run-tests` ligado, instale em uma VM e repita o roteiro abaixo antes de levar ao `emi-custom`.

## Roteiro de teste em uma VM Windows

1. Instale e confira ícone, nome, atalhos, Programas e Recursos e bandeja.
2. Com o app aberto, execute-o de novo: deve focar a janela existente.
3. Confirme a conexão com o servidor da EMI.
4. Teste senha certa, senha errada e o bloqueio após 5 erros, reiniciando o app durante o bloqueio.
5. Confirme que nenhum aviso de atualização aparece.
6. Desinstale e confira que atalhos e pasta do Menu Iniciar sumiram.

## Limites conhecidos

- **A trava protege a interface, não o sistema.** Ainda é possível trocar o servidor renomeando o executável (`host=...,key=...`, sem privilégio em uso portátil), com `--import-config`, com `--config` e `--option` (elevado), ou editando o arquivo TOML de configuração. Um servidor de API também pode enviar configurações.
- **O hash pode ser extraído do binário** e atacado offline, porque SHA-256 sem sal é rápido. Senha curta pode ser descoberta.
- **O bloqueio de 5 minutos fica em um arquivo do usuário**, que pode apagá-lo. Mudar o relógio do Windows também interfere.
- **Coexistência com o RustDesk oficial não é possível.** O serviço, a pasta e a configuração são os mesmos, então este build substitui a instalação oficial. Um servidor já salvo tem prioridade sobre o padrão do build.
- **Não instale por cima de um RustDesk instalado via MSI.** O instalador recusa esse caso; remova o MSI antes.
- **Nomes internos continuam visíveis** em lugares técnicos: pasta `C:\Program Files\RustDesk`, processo `rustdesk.exe`, regras de firewall, impressora virtual e links `rustdesk://`.
- **A tela Sobre mantém os links de privacidade e site do RustDesk**, assim como textos que citam `rustdesk.com`.
- **Só Windows x64.** O Web não executa a verificação em Rust, portanto a senha é sempre negada ali. O mobile não tem trava.

## Correções de segurança pendentes

A base é a tag 1.5.1. O `master` do upstream tinha, quando este fork foi criado, 27 commits a mais, incluindo:

- #16755: autenticação do servidor IPC no Windows
- #16885: ajustes de revisão do IPC no Windows
- #16858: relay só para endereços oferecidos pelo peer
- #16871: junção temporária do instalador do Windows

**Avalie aplicá-las antes de distribuir**, ou migre para a próxima tag estável que as inclua.
