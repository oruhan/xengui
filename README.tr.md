# XenGui

[English](README.md) | **Türkçe**

[![Crates.io](https://img.shields.io/crates/v/xengui.svg)](https://crates.io/crates/xengui)
[![Dokümantasyon](https://docs.rs/xengui/badge.svg)](https://docs.rs/xengui)
[![Rust 1.92+](https://img.shields.io/badge/rust-1.92%2B-blue.svg)](https://www.rust-lang.org)
[![Lisans: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

XenGui, Rust için retained-mode bir GUI araç takımıdır. Hook tabanlı bir component modelini,
[`taffy`](https://github.com/DioxusLabs/taffy) üzerinden Flexbox ve Grid layout'u ve
[`wgpu`](https://github.com/gfx-rs/wgpu) render backend'ini bir araya getirir. Aynı widget ve state
API'leri native masaüstü uygulamalarını ve WebAssembly'yi hedefler.

[Canlı showcase](https://xengui.vercel.app/showcase) · [Rehberler](https://xengui.vercel.app/docs) · [API referansı](https://docs.rs/xengui) · [Sorunlar](https://github.com/randseas/xengui/issues)

> [!WARNING]
> XenGui aktif olarak geliştirilmektedir. Public API'ler 1.0'dan önce değişebilir; crate
> sürümlerini sabitleyin ve üretim uygulamalarını yükseltmeden önce sürüm notlarını inceleyin.

## Showcase

[Canlı rota](https://xengui.vercel.app/showcase) etkileşimlidir; navigasyon, metin girdisi,
odaklanabilir kontroller, responsive layout, tema rolleri ve state güncellemeleri incelenebilir.
Placeholder performans telemetrisi içermez. Native referans ekranı
[`apps/showcase`](apps/showcase) altında bulunur.

```bash
cargo run -p xengui-showcase
```

## Hızlı başlangıç

Bir binary crate oluşturun ve runtime, widget ve renderer crate'lerini ekleyin:

```toml
[dependencies]
xengui = "0.2.8"
xenframe = "0.1.2"
xengui-wgpu = "0.1.2"
```

Bu küçük odak panosu; layout, state, interaction, styling, metin girdisi, checkbox, progress ve
button kullanımını kolayca çalıştırılabilir bir örnek içinde gösterir:

```rust
use xenframe::{App, AppConfig};
use xengui::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new(AppConfig {
        title: "Focus Board".into(),
        width: 760,
        height: 520,
        ..Default::default()
    });

    app.render(|| {
        let (done, set_done) = use_state(false);
        Box::new(
            Column::new()
                .padding(Edges::all(24.0))
                .gap(0.0, 16.0)
                .child(Label::new().label("Today").font_size(28.0))
                .child(
                    Row::new()
                        .align_items(Align::Center)
                        .gap(12.0, 0.0)
                        .child(
                            Checkbox::new()
                                .checked(done)
                                .on_change(move |value, _| set_done.set(value)),
                        )
                        .child(Label::new().label("Ship a polished XenGui screen")),
                )
                .child(ProgressBar::new().value(if done { 1.0 } else { 0.5 })),
        )
    });

    app.run()?;
    Ok(())
}
```

`cargo run` ile çalıştırın. Repository sürümü şu komutla kullanılabilir:

```bash
cargo run -p xengui-quickstart
```

## XenGui neler sağlar

- Component, hook, effect, resource ve context destekli retained widget kimliği.
- Flexbox, CSS Grid, responsive değerler, scrolling ve bölünmüş panel layout'ları.
- Hover, focus, pressed ve disabled durumları için tema rolleri ve interaction stilleri.
- Metin, form, görsel, SVG, navigasyon, menü, tablo ve overlay kontrolleri.
- `winit` üzerinden native pencere ve input yönetimi ile WebAssembly tarayıcı hedefi.
- Runtime, rendering, routing, animasyon, clipboard, ses, SVG ve ikonlar için odaklı crate'ler.

Renderer'ın doğrulanmış allocation ve upload davranışı
[Rendering performance](docs/rendering-performance.md) belgesinde açıklanır.

## Performans karşılaştırmaları

Repository, tam layout ve paint orchestration için tekrar üretilebilir bir CPU benchmark'ı içerir.
Karşılaştırma script'i iki Git revision'ını detached worktree'lerde build eder, aynı release iş yükünü
çalıştırır, frame başına medyan nanosaniyeyi raporlar ve yapılandırılmış regresyon bütçesi aşılırsa
başarısız olur. Sentetik FPS veya frame-time verisi üretmez ya da commit etmez.

```bash
./scripts/compare-performance.sh HEAD^ HEAD 10
```

Sonuçlar `artifacts/performance-comparison.md`, `.jsonl` ve ölçülmüş `.svg` grafik dosyalarına
yazılır. Repository [en güncel ölçülmüş karşılaştırmayı](artifacts/performance-comparison.md) içerir.
Pull request'ler aynı karşılaştırmayı CI içinde çalıştırır ve sonuçları artifact olarak yükler.

## Repository haritası

| Paket | Sorumluluk |
| --- | --- |
| [`xengui`](crates/xengui) | Widget ağacı, hook'lar, layout, styling, input ve reconciliation. |
| [`xenframe`](crates/xenframe) | Pencere yaşam döngüsü, event loop, IME, temalar ve tarayıcı entegrasyonu. |
| [`xengui-wgpu`](crates/xengui-wgpu) | `wgpu` renderer'ı ve pencere surface entegrasyonu. |
| [`xen-router`](crates/xen-router) | Client-side routing ve tarayıcı History API senkronizasyonu. |
| [`xengui-icons`](crates/xengui-icons) | Gömülü Material Symbols fontu ve codepoint'leri. |
| [`xengui-cli`](crates/xengui-cli) | Workspace kontrolleri, sürümleme, tanılama ve yayın araçları. |

Çalıştırılabilir uygulamalar [`apps`](apps) altında yer alır; görev odaklı dokümantasyon
[xengui.vercel.app/docs](https://xengui.vercel.app/docs) adresindedir, docs.rs ise API referansı olarak kalır.

## Geliştirme

Website'i yerelde çalıştırın:

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
cd apps/xengui_website
trunk serve --open
```

Tüm workspace kalite kontrollerini çalıştırın:

```bash
cargo xtask quality
```

Görsel değişikliklerden sonra native uygulama yakalamalarını yeniden üretin:

```bash
./scripts/capture-readme-assets.sh
```

CI, ilgili UI değişikliklerinde aynı doğrudan pencere yakalama komutunu çalıştırır. Pull request'ler
commit edilmiş yakalamalar güncel değilse başarısız olur; main branch değişen yakalamaları otomatik
olarak yeniler.

## Lisans

[Apache License 2.0](LICENSE) kapsamında lisanslanmıştır.
