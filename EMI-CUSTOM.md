# EMI Acess: personalização do RustDesk

Cliente de acesso remoto da Empresa Mineira de Informática, derivado do RustDesk 1.5.1 (AGPL-3.0). Este arquivo explica o que foi alterado, como gerar os instaladores de Windows, Linux, macOS, Android e iOS, como trocar servidor, chave e senha, e como atualizar a partir do upstream.

- **Fork:** `empresamineira/emi-acess`, branch `emi-custom`
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
| `ANDROID_SIGNING_KEY` | Keystore em base64 (arquivo `ANDROID_SIGNING_KEY.base64.txt`) | Só o workflow Android falha |
| `ANDROID_ALIAS` | Alias da chave (`emi-acess`) | Idem |
| `ANDROID_KEY_STORE_PASSWORD` | Senha do keystore (`senhas.txt`) | Idem |
| `ANDROID_KEY_PASSWORD` | Senha da chave (a mesma, em PKCS12) | Idem |

### Executar

Cada plataforma tem um workflow próprio. Em **Actions > (workflow) > Run workflow**, escolha o branch `emi-custom` e, ao terminar, baixe os artifacts na página da execução.

| Workflow | Arquivo | Artifacts |
|---|---|---|
| EMI Acess Windows x64 | `emi-windows-x64.yml` | `EMI-Acess-<versão>-x86_64-install.exe` |
| EMI Acess Linux | `emi-linux.yml` | `.deb` e `.AppImage` para `x86_64` e `aarch64` |
| EMI Acess macOS | `emi-macos.yml` | `.dmg` para `aarch64` (Apple Silicon) e `x86_64` (Intel) |
| EMI Acess Android | `emi-android.yml` | `.apk` para `aarch64`, `armv7`, `x86_64` e `universal` |
| EMI Acess iOS | `emi-ios.yml` | `.ipa` sem assinatura (não instala, veja abaixo) |

A opção `run-tests` (Windows e macOS) roda os testes do EMI antes de empacotar. Ela deixa o build mais lento, mas mantenha ligada depois de qualquer alteração de código.

Observações por plataforma:

- **Windows:** o instalador **não tem assinatura de código**, então o SmartScreen e antivírus podem alertar. Para eliminar isso, é preciso um certificado de assinatura.
- **Linux:** o `.deb` serve para Debian e Ubuntu 18.04 em diante. Para o AppImage, dê permissão de execução (`chmod +x`).
- **macOS:** o app tem só assinatura ad-hoc, sem conta Apple Developer. No primeiro uso o macOS bloqueia. O usuário abre em **Ajustes do Sistema > Privacidade e Segurança > Abrir Mesmo Assim**, ou o técnico roda `xattr -dr com.apple.quarantine /Applications/RustDesk.app`. No Finder o app continua se chamando `RustDesk.app`; no Dock e na barra de menu aparece "EMI Acess".
- **Android:** use o APK `universal` quando não souber o tipo do celular. Ele usa um ID próprio (`br.com.empresamineira.emiacess`) e instala ao lado do RustDesk oficial, sem levar as configurações dele.
- **iOS:** sem conta Apple Developer (US$ 99/ano), o `.ipa` não instala em iPhone. O workflow serve para garantir que o código compila. No iOS, o app só acessa outros computadores e não pode ser acessado.

### Chave de assinatura do Android

A chave fica em `C:\emi-branding\android-keystore\` neste PC:

- `emi-acess-release.jks`: o keystore. Validade de 100 anos.
- `ANDROID_SIGNING_KEY.base64.txt`: o mesmo arquivo em base64, para colar no secret.
- `senhas.txt`: alias e senha.

**Faça backup do `.jks` e das senhas em dois lugares seguros** (cofre de senhas e mídia offline). Depois, apague o `senhas.txt` do PC. Perder a chave impede atualizar os apps já instalados: seria preciso desinstalar e instalar de novo em todos os celulares. Nunca coloque esses arquivos no repositório.

## Como trocar servidor, chave ou senha

Os três valores são gravados no binário durante a compilação. Depois de trocar um secret, gere e distribua um novo instalador.

**Servidor ou chave:** atualize `RENDEZVOUS_SERVER` e `RS_PUB_KEY`, e rode o workflow. Eles são apenas valores padrão: uma máquina que já tenha um servidor salvo nas configurações continua usando o salvo. Nesse caso, corrija pela tela de configurações ou limpe os campos, para o padrão voltar a valer.

**Senha:** gere o hash sem deixar a senha no histórico do terminal (Git Bash):

```bash
read -rs SENHA; printf '%s' "$SENHA" | sha256sum; unset SENHA
```

Copie só os 64 caracteres para o secret `EMI_SETTINGS_HASH` e rode o workflow. Use uma senha **longa e aleatória** (veja os limites conhecidos).

**Nome ou publicador:** o nome exibido aparece em `src/emi.rs` (`DISPLAY_NAME`, `PUBLISHER`), `flutter/lib/consts.dart` (`kEmiDisplayName`), `flutter/windows/runner/main.cpp`, `flutter/windows/runner/service_control.h`, `flutter/windows/runner/Runner.rc` e `libs/portable/Cargo.toml`. Os valores de `main.cpp`, `src/emi.rs` e `consts.dart` precisam ser iguais, ou segunda instância e links `rustdesk://` param de funcionar. Use só ASCII, sem `" % | < >`. Nas outras plataformas, o nome também está nos arquivos listados em "Marca nas outras plataformas".

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

### Marca nas outras plataformas

Os nomes internos também não mudaram: pacote `rustdesk` no Linux, `RustDesk.app` e bundle id no macOS, bundle id no iOS, `namespace` e pacote Kotlin no Android. A exceção é o `applicationId` do Android.

| Plataforma | Arquivo | Alteração |
|---|---|---|
| Linux | `res/rustdesk.desktop`, `res/rustdesk-link.desktop` | Nome no menu de aplicativos |
| Linux | `flutter/linux/my_application.cc` | Título inicial da janela |
| Linux | `res/rustdesk.service` | Descrição do serviço systemd |
| Linux | `build.py` (`generate_control_file`) | Mantenedor, site e descrição do `.deb` |
| Linux | `appimage/AppImageBuilder-*.yml` | Nome do AppImage |
| macOS | `flutter/macos/Runner/Info.plist` | `CFBundleName` e `CFBundleDisplayName` |
| macOS | `src/platform/privileges_scripts/install.scpt`, `uninstall.scpt` | Texto da janela de senha do administrador |
| iOS | `flutter/ios/Runner/Info.plist` | Nome na tela inicial |
| Android | `flutter/android/app/build.gradle` | `applicationId` próprio |
| Android | `AndroidManifest.xml`, `res/values/strings.xml` | Nome do app, do serviço de acessibilidade e descrições |
| Android | `MainService.kt`, `BootReceiver.kt` | Notificação, canal de notificação e aviso de inicialização |
| Celular | `flutter/lib/mobile/pages/home_page.dart` | Título da tela inicial |

### Imagens

Geradas a partir de `C:\emi-branding\logo.png` (699×699). Os ICOs do Windows têm 16, 24, 32, 48, 64, 128 e 256 px, exceto o da bandeja, que tem 32 px. As imagens das outras plataformas são geradas por `res/emi-gen-icons.py` (precisa de Pillow):

```bash
python -I res/emi-gen-icons.py C:/emi-branding/logo.png .
```

| Plataforma | Arquivos | Observação |
|---|---|---|
| Linux | `res/32x32.png`, `64x64.png`, `128x128.png`, `128x128@2x.png`, `scalable.svg` | O SVG contém um PNG embutido |
| macOS | `flutter/macos/Runner/AppIcon.icns`, `res/mac-tray-dark-x2.png` | Fundo branco arredondado |
| iOS | `flutter/ios/Runner/Assets.xcassets/AppIcon.appiconset/*.png` | Sem transparência, como a Apple exige |
| Android | `mipmap-*/ic_launcher*.png`, `ic_stat_logo.png`, `drawable-xxhdpi/floating_window.png` | O ícone de notificação é uma silhueta branca. A janela flutuante trocou o vetor por PNG |

| Arquivo | Uso |
|---|---|
| `flutter/windows/runner/resources/app_icon.ico` | Ícone do `rustdesk.exe` |
| `res/icon.ico` | Ícone do instalador |
| `res/tray-icon.ico` | Bandeja, se `icon.png` não existir |
| `flutter/assets/icon.ico` (novo) | Janela |
| `flutter/assets/icon.png` (novo) | Telas e bandeja |
| `flutter/assets/logo.png` (novo) | Logo da tela inicial |

O `.gitignore` tem a regra `*png`. Se trocar por um arquivo novo, use `git add -f`.

### Atualizações bloqueadas

Com `APP_NAME` igual a `RustDesk`, o cliente consultaria as versões oficiais, que substituiriam o build da EMI. `do_check_software_update` (`src/common.rs`) retorna sem consultar nada, e as opções de atualização ficam ocultas em `desktop_setting_page.dart`. Novas versões são distribuídas por vocês, com um novo instalador.

### Servidor pré-configurado

Antes havia `option_env!` em `libs/hbb_common/src/config.rs`, mas o upstream o removeu no commit `33d90c7` do hbb_common, deixando fixos o servidor e a chave públicos. Agora `apply_build_defaults()` (`src/emi.rs`) lê os secrets e preenche `DEFAULT_SETTINGS`. Ela é chamada no início de `load_custom_client()` (`src/common.rs`), que roda na interface e no serviço. O submódulo não foi alterado.

O endereço da API e o do relay são derivados do servidor ID.

No Android, o serviço pode subir no boot antes de a interface abrir. Por isso `Java_ffi_FFI_startServer` (`src/flutter_ffi.rs`) também chama `apply_build_defaults()`. Sem isso, o celular se registraria no servidor público até alguém abrir o app.

### Trava das configurações de rede

O clique em **ID/Relay Server** pede a senha antes de abrir o diálogo, então os valores e os botões de importar e exportar ficam protegidos. A senha é pedida a cada abertura.

| Arquivo | Alteração |
|---|---|
| `src/emi.rs` | Verificação do SHA-256 e limite de tentativas |
| `src/flutter_ffi.rs` | Chave `emi-verify-server-settings:` em `main_get_common_sync`, a ponte já existente. Não precisa rodar codegen |
| `flutter/lib/desktop/widgets/emi_server_gate.dart` (novo) | Diálogo de senha |
| `flutter/lib/desktop/pages/desktop_setting_page.dart` | Passa o clique pelo diálogo (Windows, Linux e macOS) |
| `flutter/lib/mobile/pages/settings_page.dart` | Passa pelo diálogo o item ID/Relay Server e o botão de QR code, que importa um servidor (Android e iOS) |
| `flutter/test/emi_server_gate_test.dart` (novo) | Testes |

- O hash é comparado no Rust e não chega ao Flutter.
- Sem hash válido no build, a seção fica sempre bloqueada.
- Cinco senhas erradas bloqueiam a verificação por 5 minutos. Só a contagem e o horário são gravados, na opção `emi-server-settings-lockout` do `LocalConfig`. A senha nunca é gravada.
- Em instalações, a trava de administrador do RustDesk continua valendo: primeiro o UAC (ou a senha do `sudo` no Linux), depois a senha.
- O link `rustdesk://config/...` não abre o diálogo e grava o servidor direto, mas fica desligado por padrão (opção `allow-deep-link-server-settings`). Não ative essa opção.

### Workflows

Os cinco workflows `emi-*.yml` são cópias reduzidas dos jobs de `flutter-build.yml`, que não foi alterado, pois não tem seleção de alvos. Todos verificam o branch e os secrets antes de compilar. O instalador do Windows termina em `install.exe` para abrir direto a tela de instalação.

| Workflow | Job original | Diferenças |
|---|---|---|
| `emi-windows-x64.yml` | `build-for-windows-flutter` | Só x64, sem MSI, assinatura ou release |
| `emi-linux.yml` | `build-rustdesk-linux`, `build-appimage` | Sem RPM, Arch e release. O cargo roda num container, por isso os secrets entram com `--env` |
| `emi-macos.yml` | `build-for-macOS` | Assinatura ad-hoc com `.github/scripts/sign-macos-app.sh` no lugar do certificado Apple, sem notarização |
| `emi-android.yml` | `build-rustdesk-android`, `build-rustdesk-android-universal` | Assina sempre com a chave da EMI e falha sem ela |
| `emi-ios.yml` | `build-rustdesk-ios` | Empacota o `.ipa` sem assinatura |

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
2. **Workflows:** veja o que mudou nos jobs originais e leve para cada `emi-*.yml`, principalmente `VERSION`, `FLUTTER_VERSION`, `VCPKG_COMMIT_ID`, `NDK_VERSION` e actions fixadas por hash. Atualize também `version:` em `appimage/AppImageBuilder-*.yml`, como o upstream faz:
   ```bash
   git diff <versão-antiga> <versão-nova> -- .github/workflows/flutter-build.yml
   ```
3. **Atalhos novos:** procure `.lnk` e `get_app_name` em `src/platform/windows.rs`. Atalho novo deve usar `display_name`. Serviço, processo, registro e caminhos devem continuar com `app_name`.
4. **Textos novos:** procure literais `RustDesk` novos em `flutter/lib`, `src`, `flutter/android/app/src/main` e nos `Info.plist`.
5. **Verificação de atualização e servidor:** confirme que `do_check_software_update` ainda começa com o retorno antecipado, que `load_custom_client` e `Java_ffi_FFI_startServer` ainda chamam `apply_build_defaults`, e que ninguém passou a ler servidor por outro caminho.
6. **Trava:** procure novas chamadas a `showServerSettings`, `showServerSettingsWithValue` e `ScanPage` em `flutter/lib`. Toda entrada que mostra ou troca o servidor precisa passar por `showEmiServerGate`.
7. **Ícones:** se o upstream trocar ícones, mantenha os da EMI ou rode `res/emi-gen-icons.py` de novo.
8. **Teste:** rode os workflows com `run-tests` ligado, instale e repita os roteiros abaixo antes de levar ao `emi-custom`.

## Roteiros de teste

**Em todas as plataformas:**

1. Confira ícone e nome no menu, no Dock ou na tela inicial, e na janela.
2. Confirme a conexão com o servidor da EMI.
3. Teste senha certa, senha errada e o bloqueio após 5 erros, reiniciando o app durante o bloqueio.
4. Confirme que nenhum aviso de atualização aparece.

**Windows (VM):**

1. Confira atalhos, Programas e Recursos e bandeja.
2. Com o app aberto, execute-o de novo: deve focar a janela existente.
3. Desinstale e confira que atalhos e pasta do Menu Iniciar sumiram.

**Linux (Ubuntu 22.04 ou 24.04):**

1. `sudo apt install ./EMI-Acess-<versão>-x86_64.deb`.
2. Confira o item no menu de aplicativos e `systemctl status rustdesk` (descrição "EMI Acess").
3. Teste também o AppImage em outra máquina sem o `.deb`.

**macOS:** abra o `.dmg`, arraste para Aplicativos, libere em Privacidade e Segurança, dê as permissões de Gravação de Tela e Acessibilidade, e confirme que outro computador consegue acessar o Mac.

**Android:**

1. Instale o APK com o RustDesk oficial também instalado: os dois devem coexistir.
2. Confira a notificação e o nome no serviço de acessibilidade.
3. Na tela de configurações, o item ID/Relay Server e o botão de QR code devem pedir a senha.
4. Ative "iniciar no boot", reinicie o celular sem abrir o app e confirme que ele aparece online no servidor da EMI.

## Limites conhecidos

- **A trava protege a interface, não o sistema.** Ainda é possível trocar o servidor renomeando o executável (`host=...,key=...`, sem privilégio em uso portátil), com `--import-config`, com `--config` e `--option` (elevado), ou editando o arquivo TOML de configuração. Um servidor de API também pode enviar configurações.
- **O hash pode ser extraído do binário** e atacado offline, porque SHA-256 sem sal é rápido. Senha curta pode ser descoberta.
- **O bloqueio de 5 minutos fica em um arquivo do usuário**, que pode apagá-lo. Mudar o relógio do Windows também interfere.
- **Coexistência com o RustDesk oficial não é possível no desktop.** No Windows, Linux e macOS, o serviço, a pasta e a configuração são os mesmos, então este build substitui a instalação oficial. O `.deb` tem o mesmo nome de pacote `rustdesk`, e o app do macOS se chama `RustDesk.app`. Um servidor já salvo tem prioridade sobre o padrão do build. No Android há coexistência, por causa do ID próprio.
- **Não instale por cima de um RustDesk instalado via MSI.** O instalador recusa esse caso; remova o MSI antes.
- **Nomes internos continuam visíveis** em lugares técnicos: pasta `C:\Program Files\RustDesk`, processo `rustdesk.exe`, regras de firewall, impressora virtual e links `rustdesk://`.
- **A tela Sobre mantém os links de privacidade e site do RustDesk**, assim como textos que citam `rustdesk.com`.
- **Sem Windows 32 bits e sem Web.** O Windows 32 bits usa a interface antiga (Sciter), que não tem a marca nem a trava. O Web não executa a verificação em Rust, então a senha é sempre negada ali.
- **macOS e iOS sem conta Apple.** O macOS exige a liberação manual descrita acima, e o iOS não instala.
- **Nomes internos visíveis em outras plataformas:** `RustDesk.app` no Finder, o pacote `rustdesk` no `apt`, o serviço `rustdesk.service`, o dispositivo "RustDesk UInput Keyboard" no Linux e algumas mensagens de erro do Wayland em inglês.

## Correções de segurança pendentes

A base é a tag 1.5.1. O `master` do upstream tinha, quando este fork foi criado, 27 commits a mais, incluindo:

- #16755: autenticação do servidor IPC no Windows
- #16885: ajustes de revisão do IPC no Windows
- #16858: relay só para endereços oferecidos pelo peer
- #16871: junção temporária do instalador do Windows

**Avalie aplicá-las antes de distribuir**, ou migre para a próxima tag estável que as inclua.
