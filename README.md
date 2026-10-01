# hardcoded-hunter

Desktopová aplikace v Rustu pro hledání hardcoded řetězců ve zdrojových souborech.
Obsahuje grafické rozhraní a paralelní analýzu souborů.

## Požadavky

- Rust 1.75 nebo novější (`rustc` a `cargo`)

## Sestavení

Spusťte příkaz v kořenové složce projektu:

```sh
cargo build --release
```

Výsledná binárka bude v `target/release/`:

- Windows: `hardcoded-hunter.exe`
- Linux: `hardcoded-hunter`

Sestavení proveďte na cílovém operačním systému. Křížové sestavení mezi Windows
a Linuxem vyžaduje navíc cílový Rust target a odpovídající linker.

## Android (32bit ARM)

Projekt lze zabalit jako APK pro 32bitová zařízení ARMv7. Je potřeba mít
nainstalovaný Android SDK/NDK a nástroj `cargo-apk`:

```sh
rustup target add armv7-linux-androideabi
cargo install cargo-apk
cargo apk build --lib
```

Androidová varianta dovoluje zadat cestu ke složce přímo do textového pole;
dialog pro výběr složky je dostupný pouze na desktopu. Pro optimalizovaný
release APK nastavte podpisový klíč podle konfigurace `cargo-apk`.

## Optimalizace

Profil `release` používá maximální optimalizaci, fat LTO, jeden codegen unit,
odebrání symbolů a ukončení procesu při panice. Nepoužívá `target-cpu=native`,
aby výsledná binárka zůstala kompatibilní s běžnými procesory, nejen s CPU,
na kterém byla sestavena.
