# POLICY(API 方針)

正本: RUNO の API-POLICY.md。REST 全廃・Connect・gRPC。

## REST 全廃・Connect 移行の状況 (2026-11-06)

方針の正本は RUNO の `API-POLICY.md`。常駐エンジン(`aruaru-ime-daemon`、127.0.0.1:4660)の局所API(14本)と aruaru-llm 呼び出し(1本)の状況:

- **移した**: エンジン側 `POST /aruaru_ime.v1.Ime/<Method>`(Key / Templates / Session / VocabLearn / VocabRemove / VocabList / DictAdd / DictRemove / DictList / Focus / Convert / Stats / Version の13本)。POST 以外・未知のメソッドは404。呼び出し元のうちこのリポジトリ内のもの(TSF DLL・ibus・fcitx5・macOS・`tools/e2e-smoke.sh`)を同時に新経路へ。aruaru-llm の `GET /v1/knowledge/search`(1本)は、現行Connect形 `POST /aruaru_llm.v1.Llm/KnowledgeSearch`(本文 `{}`)へ。
- **期限つき互換で残した(旧経路 /key /session /templates /vocab/* /dict/* /focus /convert /stats /version)**: すでに配布済みのクライアント(インストール済みのTSF DLL、easy-web.tokyo のオンライン画面、RPoem `open-runo-crossui` が生成するページ、Android の upstream `easy-web.tokyo/ime/api`)が使うため。**期限 2026-12-06**。期限後に `crates/aruaru-ime-daemon/src/main.rs` の旧 match 腕と、Android(`crates/aruaru-ime-android`)の経路名を削除する。
- **例外**: `GET /health`(ヘルスチェック)、`GET /ui`(画面のHTML)。
- **未了/注意**: Android は upstream(easy-web.tokyo 側)の経路が変わるまで旧経路のまま。サーバー側(open-easy-web の /ime/api)の移行は別担当。受信JSONは既存どおり `sonic_rs`(上限なしのローカル専用ループバック)。

## 落ちたら自動で起動し直す(自動復帰)方針と状況 / Auto-restart policy and status (2026-11-07)

### 日本語
- **方針**: 常駐するサービス・アプリは、`kill`(SIGTERM)・強制終了(SIGKILL)・異常終了のどれで止まっても、数秒以内に自動で起動し直す。`systemctl stop` のような**意図した停止**だけは止まったままにする。REST 全廃と同時に、全リポジトリで 調査 → 確認 → TEST → DEBUG → 開発 → TEST → DEBUG を行う。
- **VPS(実施済み、2026-11-07)**: `Restart=on-failure` だったと確認した自前のサービス30個と、`postgresql`・`php-fpm`(`Restart=no`)に、`/etc/systemd/system/<unit>.d/10-autorestart.conf`(`Restart=always`、`RestartSec=3`、`StartLimitIntervalSec=0`)を追加。元のユニットファイルは変更していない。`on-failure` は SIGTERM/`kill` を正常終了とみなして再起動しないため。
- **実機TEST**: `karu-tokyo-server`・`runo-tokyo`・`rs-ops` を SIGTERM、`rs-guard` を SIGKILL で止め、6秒後に全て `active`(新しいPID)で復帰。
- **PC(Windows)の IME**: 常駐エンジン `aruaru-ime-daemon.exe` を強制終了して約3秒で `/health` が復帰することを確認(2026-11-07)。
- **未了**: 他の PC/スマホ向けアプリ(open-backup 等)の常駐プロセスの自動復帰の確認、macOS(`KeepAlive`)・Linux 版の実機確認、各リポジトリへの個別追記(担当が REST 移行する際に実施)。
- **ユニットを追加するとき**: 新しいサービスは最初から `Restart=always`、`RestartSec=3`、`StartLimitIntervalSec=0` にする。

### English
- **Policy**: resident services and apps must restart automatically within a few seconds, whether stopped by `kill` (SIGTERM), a forced kill (SIGKILL) or a crash. Only an **intentional stop** such as `systemctl stop` stays stopped. Alongside the REST abolition, every repository gets survey → verify → TEST → DEBUG → develop → TEST → DEBUG.
- **VPS (done, 2026-11-07)**: for the 30 own services found with `Restart=on-failure`, plus `postgresql` and `php-fpm` (`Restart=no`), a drop-in `/etc/systemd/system/<unit>.d/10-autorestart.conf` (`Restart=always`, `RestartSec=3`, `StartLimitIntervalSec=0`) was added. The original unit files are unchanged. Reason: `on-failure` treats SIGTERM/`kill` as a clean exit and does not restart.
- **Live TEST**: `karu-tokyo-server`, `runo-tokyo`, `rs-ops` were stopped with SIGTERM and `rs-guard` with SIGKILL; after 6 s all were `active` with new PIDs.
- **PC (Windows) IME**: force-killing the resident engine `aruaru-ime-daemon.exe` brought `/health` back in about 3 s (2026-11-07).
- **Open**: confirming automatic recovery of other PC/phone apps (open-backup etc.), real-device checks for macOS (`KeepAlive`) and Linux, and per-repository notes (done by each assignee when they migrate REST).
- **When adding a unit**: start with `Restart=always`, `RestartSec=3`, `StartLimitIntervalSec=0`.

### 追加の実機TEST(2026-11-07)/ Additional live tests
- **日本語**: Linux(WSL Ubuntu・systemd): インストーラーと同じユニット(`Restart=always`)で `aruaru-ime-daemon` を動かし、SIGTERM・SIGKILL のどちらでも約10秒以内に別PIDで復帰、`systemctl stop` では止まったままを確認。Android 実機(moto g53y・Android 14): IME のプロセスを強制終了すると OS が再生成(PID 24034→24172)。端末の既定キーボードは試験後に Gboard へ戻した。端末に入っている版は 0.2.21 でリリースは 0.2.23(未更新)。macOS の `KeepAlive` は実機が無く、設定の静的確認のみ。
- **REST 移行の副作用で見つけた不具合を修正**: VPS の `maid-news-archive` が旧 `GET /v1/internal/news/export`(移行で削除済み)を呼んで404で失敗していたため、`open-english` の `scripts/archive-articles-to-github.sh` を `POST /openenglish.v1.Internal/NewsExport` に直して反映(成功を確認)。`daily-news-collect` は `scripts/daily-news-collect.sh` の実行権限が無く失敗していたので VPS で `chmod +x`(git 上のモードは 644 のままで、`aruaru-llm` 側で直す必要あり)。
- **English**: Linux (WSL Ubuntu, systemd): the installer's unit (`Restart=always`) brought `aruaru-ime-daemon` back with a new PID within ~10 s after both SIGTERM and SIGKILL, and `systemctl stop` stayed stopped. Android device (moto g53y, Android 14): force-killing the IME process made the OS recreate it (PID 24034→24172); the default keyboard was restored to Gboard afterwards. The installed build is 0.2.21 vs release 0.2.23 (not updated). macOS `KeepAlive` was only checked statically (no device).
- **Bugs found as side effects of the REST migration, fixed**: `maid-news-archive` on the VPS called the removed `GET /v1/internal/news/export` (404); `open-english`'s `scripts/archive-articles-to-github.sh` now uses `POST /openenglish.v1.Internal/NewsExport` and is deployed (verified). `daily-news-collect` failed because `scripts/daily-news-collect.sh` lost its exec bit; fixed with `chmod +x` on the VPS (git still records mode 644; `aruaru-llm` needs the fix).

### 2026-11-07 追記: VPS のオンライン画面エンジンを新版へ / VPS online-engine updated
- **日本語**: VPS の `aruaru-ime-server`(`/opt/aruaru-ime/aruaru-ime-daemon`)を 0.2.23 に入れ替えた(バックアップ `/root/aruaru-ime-daemon.bak`、WSLでビルド・glibc 2.39)。新経路 `POST /aruaru_ime.v1.Ime/*` と旧経路の両方が通ることを確認。`easy-web.tokyo` の `/ime/api` 中継(open-easy-web)と RPoem `open-runo-crossui` の呼び出し先は、別担当が作業中(未コミットの変更あり)のため触っていない。両者が新経路になれば旧経路を削除できる(期限 2026-12-06)。
- **English**: The VPS `aruaru-ime-server` (`/opt/aruaru-ime/aruaru-ime-daemon`) was replaced with 0.2.23 (backup `/root/aruaru-ime-daemon.bak`, built in WSL, glibc 2.39); both the new `POST /aruaru_ime.v1.Ime/*` and the old routes were verified. The `easy-web.tokyo` `/ime/api` relay (open-easy-web) and the RPoem `open-runo-crossui` callers are being changed by another session (uncommitted changes), so they were left alone. Once both use the new routes the old ones can be removed (deadline 2026-12-06).

### 2026-11-09 旧REST経路の削除 / Removal of the legacy routes
- **日本語**: 常駐エンジン(`aruaru-ime-daemon`)の旧経路(`/key` `/session` `/templates` `/vocab/*` `/dict/*` `/focus` `/convert` `/stats` `/version`)を削除し、`POST /aruaru_ime.v1.Ime/*`・`GET /health`・`GET /ui` だけにした(ユーザー指示で期限2026-12-06を前倒し)。TSF/macOS/fcitx5/ibus/Android(上流)は全て新経路を使うことをコード検索で確認。ローカル実測: 旧経路すべて404、新経路200、`/health`・`/ui` 200。**Release と VPS(エンジン・`/var/lib/aruaru-ime/downloads/VERSION`)の版揃えは未実施**(インストール済みの旧TSF DLLが壊れないか、確認してから)。
- **English**: The legacy routes of the resident engine were removed; only `POST /aruaru_ime.v1.Ime/*`, `GET /health` and `GET /ui` remain (deadline 2026-12-06 brought forward on the user's instruction). Code search confirms TSF/macOS/fcitx5/ibus/Android (upstream) all use the new routes. Local test: every old route 404, new routes 200, `/health`/`/ui` 200. **Release and VPS (engine + `downloads/VERSION`) alignment is not done yet** (pending a check that an installed older TSF DLL does not break).

## 2026-11-20 変換候補の方針 / Candidate policy
- **日本語**: 候補は画面で切り捨てない。同じ読みの語が多いもの(人名など)も、変換を押し続ければ必ず全部たどれるようにする。エンジンの辞書候補の上限は `MAX_DICT_CANDIDATES`(200)。画面側(スマホ・タブレット)は全件を横スクロールで並べ、Windows は候補窓のページ送りで全件をたどる。新しい表示先を作るときも、この「全部たどれる」を試験で確かめる。
- **English**: candidates are never cut off on screen. Readings with many spellings (given names, etc.) must be fully reachable by pressing convert repeatedly. The engine's dictionary cap is `MAX_DICT_CANDIDATES` (200). Phones/tablets list every candidate in a scrolling row; Windows pages through every candidate in its window. Any new front-end must test that every candidate is reachable.

## 2026-11-20 v0.2.28 の公開と現状・予定 / v0.2.28 released: status and plans

### 日本語
**v0.2.28 で直したこと**: 「まさひろ→正浩」「ひろし→浩」のように、同じ読みの語が多い言葉も、「変換」を押し続ければ全部の候補をたどれるようになりました(スマホのキーボードは前は12個までしか出さず、エンジンは30個で打ち切っていました)。タブレットや横向きでキーボードが出なかったり、入力欄が隠れたりする不具合も直しました。
**今できること**: Windows・macOS・Linux・Android(スマホ・タブレット)で、ローマ字から日本語へのライブ変換、ひらがな・カタカナ・英字・記号の切り替え、学習、定型文。変換エンジンはサーバー(easy-web.tokyo)でも動いています。
**これからの予定**: よく使う人名を候補の上の方に出す順位づけ、本物のタブレットでの確認、Windows インストーラーの入れ直し試験(管理者の許可「はい」はご本人が押します)。

### English
**Fixed in v0.2.28**: words with many spellings for one reading, such as 「まさひろ」→「正浩」 and 「ひろし」→「浩」, can now be reached by pressing convert repeatedly (the phone keyboard used to show only 12 candidates, and the engine stopped at 30). The keyboard also appears reliably on tablets and in landscape, without hiding the text field.
**What works now**: live romaji-to-Japanese conversion on Windows, macOS, Linux and Android (phones and tablets), switching between hiragana, katakana, Latin letters and symbols, learning, and templates. The engine also runs on the server (easy-web.tokyo).
**Planned**: ranking common given names higher, a check on a real tablet, and a full Windows installer re-install test (the owner presses "Yes" on the administrator prompt).

### 简体中文
**v0.2.28 的修复**:像"まさひろ→正浩""ひろし→浩"这样同一读音有很多写法的词,现在只要连续按"变换",就能依次看到所有候选词(以前手机键盘最多只显示12个,引擎在30个就停止)。平板和横屏时键盘不出现、挡住输入框的问题也已修好。
**现在能做的**:在 Windows、macOS、Linux、Android(手机和平板)上把罗马字实时转换成日语,切换平假名、片假名、英文字母和符号,学习用词和常用语。转换引擎也在服务器(easy-web.tokyo)上运行。
**接下来的计划**:把常用人名排得更靠前、在真实平板上确认、完整测试 Windows 安装程序的重新安装(管理员确认需要本人点击"是")。

### 繁體中文
**v0.2.28 的修正**:像「まさひろ→正浩」「ひろし→浩」這種同一讀音有許多寫法的詞,現在只要連續按「變換」,就能依序看到所有候選字(以前手機鍵盤最多只顯示12個,引擎到30個就停止)。平板與橫向時鍵盤不出現、擋住輸入框的問題也已修好。
**現在能做的**:在 Windows、macOS、Linux、Android(手機與平板)上將羅馬字即時轉換成日文,切換平假名、片假名、英文字母與符號,學習用詞與常用語。轉換引擎也在伺服器(easy-web.tokyo)上執行。
**接下來的計畫**:將常用人名排得更前面、在實際平板上確認、完整測試 Windows 安裝程式的重新安裝(管理員確認需由本人按「是」)。

### 한국어
**v0.2.28에서 고친 점**: "まさひろ→正浩", "ひろし→浩"처럼 읽는 법은 같고 표기가 많은 단어도, 이제 "변환"을 계속 누르면 모든 후보를 차례로 볼 수 있습니다(예전에는 휴대폰 키보드가 12개까지만 보여 주고, 엔진은 30개에서 멈췄습니다). 태블릿이나 가로 화면에서 키보드가 안 나오거나 입력란을 가리던 문제도 고쳤습니다.
**지금 할 수 있는 것**: Windows, macOS, Linux, Android(휴대폰·태블릿)에서 로마자를 일본어로 실시간 변환하고, 히라가나·가타카나·영문자·기호를 전환하며, 학습과 상용구를 씁니다. 변환 엔진은 서버(easy-web.tokyo)에서도 동작합니다.
**앞으로의 계획**: 자주 쓰는 사람 이름을 위쪽에 보여 주는 순위 개선, 실제 태블릿에서 확인, Windows 설치 프로그램 재설치 전체 시험(관리자 확인의 "예"는 본인이 누릅니다).

### Deutsch
**Behoben in v0.2.28**: Wörter mit vielen Schreibweisen für eine Lesung, etwa 「まさひろ」→「正浩」 und 「ひろし」→「浩」, lassen sich jetzt erreichen, indem man „Umwandeln“ mehrmals drückt (die Handy-Tastatur zeigte bisher nur 12 Vorschläge, und die Engine hörte bei 30 auf). Auf Tablets und im Querformat erscheint die Tastatur jetzt zuverlässig und verdeckt das Eingabefeld nicht mehr.
**Was jetzt funktioniert**: Live-Umwandlung von Romaji in Japanisch unter Windows, macOS, Linux und Android (Handy und Tablet), Wechsel zwischen Hiragana, Katakana, lateinischen Buchstaben und Symbolen, Lernen und Textbausteine. Die Engine läuft auch auf dem Server (easy-web.tokyo).
**Geplant**: häufige Vornamen weiter oben anzeigen, Prüfung auf einem echten Tablet, ein vollständiger Neuinstallations-Test des Windows-Installers (das „Ja“ bei der Administrator-Abfrage drückt der Besitzer selbst).

### Français
**Corrigé dans la v0.2.28** : les mots ayant de nombreuses graphies pour une même lecture, comme 「まさひろ」→「正浩」 et 「ひろし」→「浩」, sont désormais accessibles en appuyant plusieurs fois sur « Convertir » (le clavier du téléphone n'affichait que 12 suggestions et le moteur s'arrêtait à 30). Le clavier apparaît aussi correctement sur tablette et en mode paysage, sans cacher le champ de saisie.
**Ce qui fonctionne aujourd'hui** : conversion en direct du romaji vers le japonais sur Windows, macOS, Linux et Android (téléphones et tablettes), passage entre hiragana, katakana, lettres latines et symboles, apprentissage et modèles de texte. Le moteur tourne aussi sur le serveur (easy-web.tokyo).
**Prévu** : mieux classer les prénoms courants, vérifier sur une vraie tablette, et un test complet de réinstallation du programme d'installation Windows (c'est le propriétaire qui clique sur « Oui » à la demande d'administrateur).

### Español
**Corregido en v0.2.28**: las palabras con muchas escrituras para una misma lectura, como 「まさひろ」→「正浩」 y 「ひろし」→「浩」, ahora se alcanzan pulsando «Convertir» varias veces (antes el teclado del móvil mostraba solo 12 candidatos y el motor se detenía en 30). El teclado también aparece bien en tabletas y en horizontal, sin tapar el campo de texto.
**Lo que funciona ahora**: conversión en vivo de romaji a japonés en Windows, macOS, Linux y Android (móviles y tabletas), cambio entre hiragana, katakana, letras latinas y símbolos, aprendizaje y plantillas. El motor también funciona en el servidor (easy-web.tokyo).
**Previsto**: mostrar más arriba los nombres de pila habituales, comprobarlo en una tableta real y una prueba completa de reinstalación del instalador de Windows (el «Sí» del aviso de administrador lo pulsa el propietario).

### Português
**Corrigido na v0.2.28**: palavras com muitas grafias para a mesma leitura, como 「まさひろ」→「正浩」 e 「ひろし」→「浩」, agora são alcançadas tocando em «Converter» várias vezes (antes o teclado do celular mostrava só 12 candidatos e o motor parava em 30). O teclado também aparece corretamente em tablets e na horizontal, sem cobrir o campo de texto.
**O que funciona agora**: conversão ao vivo de romaji para japonês no Windows, macOS, Linux e Android (celulares e tablets), troca entre hiragana, katakana, letras latinas e símbolos, aprendizado e modelos de texto. O motor também roda no servidor (easy-web.tokyo).
**Previsto**: mostrar nomes próprios comuns mais acima, verificar num tablet real e um teste completo de reinstalação do instalador do Windows (o «Sim» do aviso de administrador é tocado pelo próprio dono).

### Русский
**Исправлено в v0.2.28**: слова с множеством написаний для одного чтения, например 「まさひろ」→「正浩」 и 「ひろし」→「浩」, теперь можно найти, нажимая «Преобразовать» несколько раз (раньше клавиатура телефона показывала только 12 вариантов, а движок останавливался на 30). На планшетах и в горизонтальном положении клавиатура теперь появляется надёжно и не закрывает поле ввода.
**Что работает сейчас**: преобразование ромадзи в японский текст на лету в Windows, macOS, Linux и Android (телефоны и планшеты), переключение между хираганой, катаканой, латиницей и символами, обучение и шаблоны текста. Движок также работает на сервере (easy-web.tokyo).
**В планах**: поднимать частые личные имена выше в списке, проверка на настоящем планшете и полный тест переустановки Windows-установщика (кнопку «Да» в запросе администратора нажимает сам владелец).

(日本語・英語以外はAI翻訳で、ネイティブの確認前です / Languages other than Japanese and English are AI translations, not yet checked by native speakers)

## 2026-11-20 人名の候補の改善(v0.2.29) / Better given-name candidates (v0.2.29)
(日本語・英語以外はAI翻訳で、ネイティブの確認前です / Languages other than Japanese and English are AI translations, not yet checked by native speakers)

### 日本語
- **よく使う名前を上の方に**: 「まさひろ→正浩」「ひろし→浩」のような、よく使う名前の表記を、いちばん上の候補のすぐ次(上位3つ)に出します。前は15番目あたりでした(`crates/aruaru-ime-core/src/names.rs`)。
- **辞書に無い名前も必ず出す**: 辞書に無い組み合わせの名前も、名前によく使う読みの部品(まさ+ひろ、たか+のぶ)をつないで表記を作り、候補に必ず入れます。例: 「たかのぶ→隆信」。
- **確かめたこと**: 試験207件すべて成功。このPCに v0.2.29 のインストーラーで入れ直し(UACはご本人が許可)、正浩2番目・浩3番目・隆信2番目を確認。VPS のエンジンも差し替え(元は `aruaru-ime-daemon.bak-20261120c`)、公開 API で同じ結果、エラー0件。スマホのキーボードは VPS で変換するので、もう届いています。
- **インストール試験で分かったこと**: アンインストールは成功。その直後の再インストールは、Windows が「再起動が必要」と判断して止まりました(使用中の IME の DLL は再起動しないと消えないため)。次のサインインで自動で入れる予約をしました。
- **これから**: 名前の一覧を増やす、Windows の再起動後の自動インストールの確認、v0.2.29 の公開(Release)。

### English
- **Common names come first**: common spellings of given names, such as 「まさひろ」→「正浩」 and 「ひろし」→「浩」, now appear right after the top candidate (top 3). Before, they were around 15th (`crates/aruaru-ime-core/src/names.rs`).
- **Names not in the dictionary always appear**: for a name the dictionary lacks, the engine joins common name parts (まさ+ひろ, たか+のぶ) and always adds the result. Example: 「たかのぶ」→「隆信」.
- **Checked**: all 207 tests pass. Re-installed with the v0.2.29 installer on this PC (the owner approved UAC): 正浩 2nd, 浩 3rd, 隆信 2nd. The VPS engine was swapped too (old one kept as `aruaru-ime-daemon.bak-20261120c`); the public API gives the same result with no errors. The phone keyboard converts on the VPS, so it already has the fix.
- **Installer test finding**: uninstall works. A re-install right after it stopped because Windows wants a restart (an IME DLL in use can only be removed after a restart). The install is scheduled to run at the next sign-in.
- **Next**: a longer name list, checking the after-restart install, and publishing v0.2.29 as a Release.

### 简体中文
常用人名的写法(まさひろ→正浩、ひろし→浩)现在排在第一个候选之后的前三位。词典里没有的名字,也会用常见的名字部件(まさ+ひろ、たか+のぶ)组合出来,一定出现在候选里(例:たかのぶ→隆信)。207项测试全部通过,已在本机安装程序和服务器上确认。卸载后立即重新安装需要重启 Windows,已预约在下次登录时自动安装。
### 繁體中文
常用人名的寫法(まさひろ→正浩、ひろし→浩)現在排在第一個候選之後的前三位。字典裡沒有的名字,也會用常見的名字部件(まさ+ひろ、たか+のぶ)組合出來,一定出現在候選中(例:たかのぶ→隆信)。207項測試全部通過,已在本機安裝程式與伺服器上確認。解除安裝後立即重新安裝需要重新啟動 Windows,已預約在下次登入時自動安裝。
### 한국어
자주 쓰는 이름 표기(まさひろ→正浩, ひろし→浩)가 이제 첫 후보 바로 다음(상위 3개)에 나옵니다. 사전에 없는 이름도 자주 쓰는 이름 부품(まさ+ひろ, たか+のぶ)을 이어 표기를 만들어 반드시 후보에 넣습니다(예: たかのぶ→隆信). 시험 207건 모두 통과, 이 PC의 설치 프로그램과 서버에서 확인했습니다. 제거 직후 재설치는 Windows 재시작이 필요해, 다음 로그인 때 자동 설치하도록 예약했습니다.
### Deutsch
Häufige Schreibweisen von Vornamen (まさひろ→正浩, ひろし→浩) stehen jetzt direkt nach dem ersten Vorschlag (unter den ersten drei). Namen, die nicht im Wörterbuch stehen, werden aus üblichen Namensteilen (まさ+ひろ, たか+のぶ) gebildet und erscheinen immer (z. B. たかのぶ→隆信). Alle 207 Tests bestehen; mit dem Installer auf diesem PC und auf dem Server geprüft. Eine Neuinstallation direkt nach der Deinstallation braucht einen Windows-Neustart; sie ist für die nächste Anmeldung geplant.
### Français
Les graphies courantes des prénoms (まさひろ→正浩, ひろし→浩) apparaissent maintenant juste après la première suggestion (parmi les trois premières). Les prénoms absents du dictionnaire sont formés à partir d'éléments de prénom courants (まさ+ひろ, たか+のぶ) et apparaissent toujours (ex. たかのぶ→隆信). Les 207 tests réussissent ; vérifié avec l'installateur sur ce PC et sur le serveur. Une réinstallation juste après la désinstallation demande un redémarrage de Windows ; elle est programmée à la prochaine connexion.
### Español
Las escrituras habituales de nombres de pila (まさひろ→正浩, ひろし→浩) aparecen ahora justo después del primer candidato (entre los tres primeros). Los nombres que no están en el diccionario se forman con partes de nombre habituales (まさ+ひろ, たか+のぶ) y siempre aparecen (p. ej. たかのぶ→隆信). Pasan las 207 pruebas; comprobado con el instalador en este PC y en el servidor. Reinstalar justo después de desinstalar requiere reiniciar Windows; quedó programado para el próximo inicio de sesión.
### Português
As grafias comuns de nomes próprios (まさひろ→正浩, ひろし→浩) agora aparecem logo após o primeiro candidato (entre os três primeiros). Nomes que não estão no dicionário são formados com partes comuns de nomes (まさ+ひろ, たか+のぶ) e sempre aparecem (ex.: たかのぶ→隆信). Os 207 testes passam; verificado com o instalador neste PC e no servidor. Reinstalar logo após desinstalar exige reiniciar o Windows; ficou agendado para o próximo login.
### Русский
Частые написания личных имён (まさひろ→正浩, ひろし→浩) теперь стоят сразу после первого варианта (в первой тройке). Имена, которых нет в словаре, собираются из частых частей имён (まさ+ひろ, たか+のぶ) и всегда появляются (например, たかのぶ→隆信). Все 207 тестов проходят; проверено установщиком на этом ПК и на сервере. Переустановка сразу после удаления требует перезагрузки Windows; она запланирована на следующий вход в систему.

## 2026-11-20 追記: 人名の一覧を拡充 / More given names
(日本語・英語以外はAI翻訳、ネイティブ確認前 / non-JA/EN parts are AI translations, not native-checked)
- **日本語**: よく使う名前の一覧を約60件から約110件に増やしました(あきひろ・みのる・ひろみ・たけし など)。よく使う名前の読みでは、長い地名の予測(ひろし→広島県)を名前の前に割り込ませないようにし、「ひろし→浩」は2番目になりました。試験207件成功、VPS のエンジンを差し替え(元は `aruaru-ime-daemon.bak-20261120d`)、公開 API で確認、エラー0件。Windows・スマホのアプリへの反映は次のリリースで入ります(スマホのキーボードは VPS で変換するので、すでに反映済み)。
- **English**: the common-name list grew from about 60 to about 110 readings (あきひろ, みのる, ひろみ, たけし, ...). For common name readings, long place-name predictions (ひろし→広島県) no longer jump ahead of the names, so 浩 is now 2nd for ひろし. 207 tests pass; the VPS engine was swapped (old one kept as `aruaru-ime-daemon.bak-20261120d`) and checked through the public API with no errors. The phone keyboard converts on the VPS, so it already has this; the Windows app gets it with the next release.
- **简体中文**: 常用人名列表从约60个增加到约110个;常用人名的读音不再被长地名预测(ひろし→広島県)插队,"ひろし→浩"现在排第2。
- **繁體中文**: 常用人名清單從約60個增加到約110個;常用人名的讀音不再被長地名預測(ひろし→広島県)插隊,「ひろし→浩」現在排第2。
- **한국어**: 자주 쓰는 이름 목록을 약 60개에서 약 110개로 늘렸습니다. 이름 읽기에서 긴 지명 예측(ひろし→広島県)이 끼어들지 않아 "ひろし→浩"가 2번째가 되었습니다.
- **Deutsch**: Die Liste häufiger Vornamen wuchs von etwa 60 auf etwa 110 Lesungen; lange Ortsnamen-Vorschläge (ひろし→広島県) drängen sich nicht mehr vor die Namen, 浩 steht jetzt an 2. Stelle.
- **Français**: la liste des prénoms courants passe d'environ 60 à environ 110 lectures ; les longues prédictions de noms de lieux (ひろし→広島県) ne passent plus devant les prénoms, 浩 est désormais 2ᵉ.
- **Español**: la lista de nombres habituales pasó de unos 60 a unas 110 lecturas; las predicciones largas de lugares (ひろし→広島県) ya no se adelantan a los nombres, y 浩 queda 2.º.
- **Português**: a lista de nomes comuns passou de cerca de 60 para cerca de 110 leituras; previsões longas de lugares (ひろし→広島県) não passam mais à frente dos nomes, e 浩 agora é o 2.º.
- **Русский**: список частых имён вырос примерно с 60 до 110 чтений; длинные подсказки географических названий (ひろし→広島県) больше не встают перед именами, 浩 теперь на 2-м месте.

## 2026-11-21 v0.2.30 公開 / v0.2.30 released
- **日本語**: よく使う人名の一覧(約110件)と「ひろし→浩」で地名が割り込まない修正を、Windows・macOS・Linux・Androidの配布物に入れました。Release・VPSの配布ファイル・紹介ページはv0.2.30です。
- **English**: The expanded given-name list (about 110 readings) and the fix that stops place names cutting in (e.g. ひろし→浩) now ship in the Windows, macOS, Linux and Android builds. The Release, VPS downloads and intro page are at v0.2.30.
- **简体中文**: 常用人名列表(约110个)以及"ひろし→浩"不再被地名插队的修复,已加入 Windows、macOS、Linux、Android 安装包。
- **繁體中文**: 常用人名清單(約110個)以及「ひろし→浩」不再被地名插隊的修正,已加入 Windows、macOS、Linux、Android 安裝檔。
- **한국어**: 자주 쓰는 이름 목록(약 110개)과 "ひろし→浩"에서 지명이 끼어들지 않게 한 수정을 Windows·macOS·Linux·Android 배포판에 넣었습니다.
- **Deutsch**: Die erweiterte Vornamenliste (etwa 110 Lesungen) und die Korrektur, dass sich Ortsnamen nicht mehr vordrängen (z. B. ひろし→浩), sind jetzt in den Builds für Windows, macOS, Linux und Android.
- **Français**: La liste élargie de prénoms (environ 110 lectures) et la correction empêchant les noms de lieux de s'intercaler (ex. ひろし→浩) sont désormais livrées pour Windows, macOS, Linux et Android.
- **Español**: La lista ampliada de nombres de pila (unas 110 lecturas) y la corrección para que los topónimos no se cuelen (p. ej. ひろし→浩) ya vienen en las versiones de Windows, macOS, Linux y Android.
- **Português**: A lista ampliada de nomes próprios (cerca de 110 leituras) e a correção para que nomes de lugares não se intrometam (ex.: ひろし→浩) já vêm nas versões para Windows, macOS, Linux e Android.
- **Русский**: Расширенный список личных имён (около 110 чтений) и исправление, при котором названия мест больше не вклиниваются (например, ひろし→浩), теперь входят в сборки для Windows, macOS, Linux и Android.
(日本語・英語以外はAI翻訳 / non-JA/EN lines are AI translations)
