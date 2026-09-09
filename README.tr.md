# XenGui

[English](README.md) | **Türkçe**

[![Crates.io](https://img.shields.io/crates/v/xengui.svg)](https://crates.io/crates/xengui)
[![Dokümantasyon](https://docs.rs/xengui/badge.svg)](https://docs.rs/xengui)
[![Rust 1.92+](https://img.shields.io/badge/rust-1.92%2B-blue.svg)](https://www.rust-lang.org)
[![Lisans: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

XenGui, Rust ile yazılmış retained-mode bir GUI araç takımıdır. Hook tabanlı bir bileşen modelini, [`taffy`](https://github.com/DioxusLabs/taffy) üzerinden Flexbox ve Grid yerleşimini ve [`wgpu`](https://github.com/gfx-rs/wgpu) üzerinden GPU render desteğini bir araya getirir. Aynı uygulama kodu masaüstünü ve WebAssembly'yi hedefleyebilir.

[Canlı demo](https://xengui.vercel.app) | [Dokümantasyon](https://xengui.vercel.app/docs) | [API referansı](https://docs.rs/xengui) | [Sorun takipçisi](https://github.com/randseas/xengui/issues)

> [!WARNING]
> XenGui aktif olarak geliştirilmektedir. Public API'ler 1.0 sürümünden önce değişebilir; üretim uygulamalarını yükseltmeden önce bağımlılık sürümlerini sabitleyin ve sürüm notlarını inceleyin.

## Öne çıkanlar

- `component`, `use_state`, effect, resource ve context destekli retained widget ağacı.
- Flexbox, CSS Grid, responsive değerler, scrolling ve bölünmüş panel yerleşimleri.
- Transition ve filter dahil olmak üzere declarative temalar ve etkileşim durumuna özel stiller.
- Metin, form, görsel, SVG, navigasyon, menü, tablo ve overlay için yerleşik kontroller.
- Dikdörtgen, metin, görsel, SVG üçgenleri, filter ve shadow işlemleri için yeniden kullanılabilir frame staging destekli instanced ve batched `wgpu` pipeline'ları.
- `winit` üzerinden native pencere ve input yönetimi; WebAssembly üzerinden tarayıcı desteği.
- Rendering, runtime, routing, animasyon, clipboard, ses, SVG ve ikon işlevlerini birbirinden ayıran odaklı crate'ler.

## Mimari

| Paket | Görev |
| --- | --- |
| [`xengui`](crates/xengui) | Platformdan bağımsız widget ağacı, hook'lar, layout, styling ve reconciliation. |
| [`xenframe`](crates/xenframe) | Pencere oluşturma, event loop, input, IME, tema ve tarayıcı entegrasyonu. |
| [`xengui-wgpu`](crates/xengui-wgpu) | GPU render backend'i ve pencere renderer'ı. |
| [`xen-router`](crates/xen-router) | Tarayıcı History API senkronizasyonuna sahip istemci taraflı routing. |
| [`xen-router-build`](crates/xen-router-build) | Dosya tabanlı route'lar için build-time generator. |
| [`xen-animation`](crates/xen-animation) | Framework'ten bağımsız transition ve easing işlevleri. |
| [`xen-clipboard`](crates/xen-clipboard) | Asenkron metin clipboard soyutlaması. |
| [`xen-audio`](crates/xen-audio) | Framework'ten bağımsız yerel ses oynatma soyutlaması. |
| [`xen-svg`](crates/xen-svg) | SVG parsing ve triangle tessellation. |
| [`xengui-icons`](crates/xengui-icons) | Gömülü Material Symbols variable icon fontu ve codepoint'leri. |
| [`xengui-cli`](crates/xengui-cli) | Workspace geliştirme, sürümleme, Git, tanılama ve yayın araçları. |

Çalıştırılabilir uygulamalar [`apps`](apps), belirli özelliklere odaklanan örnekler ise [`examples`](examples) dizininde bulunur.

Renderer'ın allocation ve upload modeli [Rendering performance](docs/rendering-performance.md) belgesinde açıklanmıştır.

## Gereksinimler

- Workspace MSRV tanımına uygun olarak Rust 1.92 veya üzeri.
- `wgpu` tarafından desteklenen bir grafik adaptörü ve sürücü.
- Tarayıcı build'leri için [Trunk](https://trunk-rs.github.io/trunk/) ve `wasm32-unknown-unknown` Rust target'ı.
- Workspace'in tamamını derlemek için platformun ses geliştirme paketi; Linux'ta `xen-audio`, `pkg-config` tarafından bulunabilen ALSA geliştirme dosyalarını gerektirir.

## Hızlı başlangıç

Bir binary crate oluşturun ve uygulama runtime bağımlılıklarını ekleyin:

```toml
[dependencies]
xengui = "0.2.8"
xenframe = "0.1.2"
xengui-wgpu = "0.1.2"
```

```rust
use xenframe::{App, AppConfig};
use xengui::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new(AppConfig {
        title: "Counter".into(),
        width: 640,
        height: 480,
        ..Default::default()
    });

    app.render(|| {
        let (count, set_count) = use_state(0_i32);

        Box::new(
            Column::new()
                .width(pct!(100))
                .height(pct!(100))
                .align_items(Align::Center)
                .justify_content(JustifyContent::Center)
                .gap(0, 12)
                .child(Label::new().label(format!("Count: {count}")))
                .child(
                    Button::new()
                        .label("Increment")
                        .on_click(move |_| set_count.update(|value| *value += 1)),
                ),
        )
    });

    app.run()?;
    Ok(())
}
```

Uygulamayı `cargo run` ile çalıştırın.

## Workspace'i çalıştırma

Repository root dizininden:

```bash
cargo run -p widgets-catalog
```

Diğer kullanışlı hedefler arasında `animation-example`, `filters-example`, `layout-example`, `router-example`, `scroll-example`, `settings-app`, `pearl` ve `xengui_website` bulunur.

Tarayıcı build'i için:

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
cd examples/widgets_catalog
trunk serve --open
```

Trunk yerel bir development build sunar ve kaynak dosyalar değiştiğinde yeniden build alır.

## Geliştirme

Standart kalite kontrollerini repository root dizininden çalıştırın:

```bash
cargo fmt --all --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

GPU kullanılabilirliği ve target'a özel bağımlılıklar native veya WebAssembly kontrollerini etkileyebilir. Rendering kodunu değiştirirken hem native bir örneği hem de tarayıcı build'ini test edin.

Linux'ta `alsa.pc` dosyasının bulunamadığını belirten bir hata, `xen-audio` veya `pearl` testlerinden önce dağıtımın ALSA geliştirme paketinin ve `pkg-config` aracının kurulması gerektiği anlamına gelir.

Katkılar [issue'lar](https://github.com/randseas/xengui/issues) ve pull request'ler aracılığıyla kabul edilir. Kapsamlı değişikliklerde tasarımın tartışılabilmesi için önce bir issue açın. Davranış değişikliklerine test veya yeniden üretilebilir bir örnek ekleyin.

## Lisans

[Apache License 2.0](LICENSE) kapsamında lisanslanmıştır.
