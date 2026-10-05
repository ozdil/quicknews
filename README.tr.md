# QuickNews

[![Omarchy Verified Plugin](https://img.shields.io/badge/Omarchy-Verified_Plugin-22c55e?style=for-the-badge&logo=omarchy)](https://github.com/ozdil)
[![Buy Me A Coffee](https://img.shields.io/badge/Buy_Me_A_Coffee-Destek_Ol-FFDD00?style=for-the-badge&logo=buy-me-a-coffee&logoColor=black)](https://buymeacoffee.com/ozdil)

Rust ve Quickshell ile güçlendirilmiş, yüksek performanslı, açık kaynaklı, reklamsız, dikkat dağıtmayan ve saf metin odaklı haber okuyucu.

QuickNews; yazılımcılar, araştırmacılar ve minimalistler için tasarlanmış ödünsüz bir okuma deneyimi sunar. Tüm görsel karmaşayı, tık tuzağı (clickbait) başlıkları, rahatsız edici banner reklamları, izleme betiklerini ve çerez pencerelerini temizleyerek geriye yalnızca temiz tipografi ve saf haber metnini bırakır.

Omarchy Linux masaüstü ekosistemi ve modern Linux dağıtımları için geliştirilmiş olan QuickNews, bağımsız Rust çekirdeği ile modüler bir CLI aracı ve Wayland tabanlı yerel bir grafik arayüzü sunar.

[English](README.md) • [Türkçe](README.tr.md)

---

## Öne Çıkan Özellikler

- **Açık Kaynak ve Gizlilik Öncelikli:** MIT Lisansı altında tamamen açık kaynaklıdır. Sıfır telemetri, sıfır arka plan izleme ve harici bulut bağımlılığı olmadan tamamen yerel olarak çalışır.
- **%100 Reklamsız, Takipçisiz ve Görselsiz:** DOM düzeyinde filtreleme ile reklam banner'ları, çerez onay pencereleri, sponsorlu bağlantılar, bülten abonelik modalları, satır içi görseller (`<img>`), video gömmeleri ve takip parametreleri (`utm_*`, `fbclid`, `gclid`) tamamen temizlenir.
- **Sıkı RSS / Atom XML Standart Koruması:** QuickNews modern RSS 2.0 ve Atom XML standartlarını uygular. Bir kaynak kaydedilmeden önce hedef adres test edilir. Kırık linkler, 404 sayfaları ve düz HTML yönlendirmeleri reddedilir; yalnızca çalışan ve geçerli XML akışları eklenir.
- **Doğal Dille Kaynak Keşfi:** XML linkleri aramak zorunda kalmadan akış ekleyin. `"En popüler Linux ve teknoloji sitelerini ekle"` veya `"Türkiye yerel haberlerini ekle"` şeklinde doğal dil istemi girin. Motor, bilgi tabanını tarar, aday alan adlarını doğrular ve XML akışlarını otomatik olarak listeye dahil eder.
- **Nötr 3 Maddelik Özetler:** Uzun haber metinlerini analiz ederek sakin, nesnel ve tık tuzağından arındırılmış bir başlık ile birlikte 3 maddelik hap özet sunar.
- **Ergonomik Tipografi:** Tüm panellerde `JetBrainsMono Nerd Font` varsayılandır. Göz yorgunluğunu önlemek için ideal sütun genişliği (780px) ve 1.6 satır aralığı kullanılır.
- **Canlı Sistem Teması Entegrasyonu:** Omarchy `colors.toml` yapılandırmasını anlık izler; masaüstü renk paleti değiştiğinde arayüz anında güncellenir.
- **Sertleştirilmiş Güvenlik Mimarisi (HANCORE Standartları):**
  - **SSRF Savunması:** Soket bağlantısı kurulmadan önce özel ağ (RFC 1918), yerel bağlantı (RFC 3927) ve geri döngü (loopback) IP adresleri engellenir.
  - **Sınırlandırılmış Veri:** Bellek şişmesini ve DoS saldırılarını önlemek için 8 MiB HTTP veri tavan sınırı uygulanır.
  - **Atomik Depolama:** Yapılandırma ve haber önbellekleri geçici atomik dosyalar (`.tmp_*`) üzerinden, katı 0600 dosya ve 0700 dizin izinleriyle yazılır. Sembolik bağlar (`symlink`) reddedilir.

---

## Mimarisi

```text
quicknews/
|-- Cargo.toml                      Rust manifest ve motor bağımlılıkları
|-- CONTRIBUTING.md                 Geliştirici kuralları ve HANCORE güvenlik standartları
|-- README.md                       İngilizce dokümantasyon
|-- README.tr.md                    Türkçe dokümantasyon
|-- PKGBUILD                        Arch Linux / Omarchy paket tanımı
|-- build.sh                        Kullanıcı alanı güvenli derleme betiği
|-- uninstall.sh                    Doğrulanmış güvenli kaldırma betiği
|-- quicknews                       Ana CLI başlatıcı ve IPC toggle betiği
|-- quicknews.desktop               XDG masaüstü kısayolu
|-- Panel.qml                       Omarchy Quickshell üst çubuk durum widget'ı
|-- src/
|   |-- lib.rs                      Çekirdek Rust kütüphane arayüzü ve senkronizasyon motoru
|   |-- main.rs                     quicknews-engine CLI ikilisi ve IPC işleyicisi
|   `-- core/
|       |-- security.rs             SSRF doğrulayıcı, atomik dosya G/Ç ve DNS koruması
|       |-- feed.rs                 RSS 2.0 / Atom XML ayrıştırıcı ve doğrulama motoru
|       |-- extractor.rs            Temiz metin çıkarma, reklam temizleme ve sayfalama
|       |-- adblock.rs              URL takipçi temizleyici ve DOM filtresi
|       |-- ai.rs                   Doğal dil kaynak çözümleme ve yerel NLP özetleyici
|       `-- storage.rs              Atomik 0600/0700 dosya depolama ve önbellek yöneticisi
`-- qml/
    |-- shell.qml                   Quickshell ShellRoot, Wayland FloatingWindow ve IPC
    |-- MainWindow.qml              Üç panelli duyarlı kullanıcı arayüzü
    |-- components/                 Arayüz bileşenleri (Kenar çubuğu, başlık listesi, okuyucu)
    `-- theme/
        |-- Theme.qml               Dinamik colors.toml paleti ve JetBrainsMono fontu
        `-- I18n.qml                Çift dilli yerelleştirme motoru (Türkçe / İngilizce)
```

---

## Kısayollar

| Kısayol | İşlev |
| :--- | :--- |
| `J` / `Aşağı Ok` | Sonraki haberi seç |
| `K` / `Yukarı Ok` | Önceki haberi seç |
| `F` | Zen / Tam ekran okuma modunu aç/kapat |
| `S` | Haberi kaydet veya yer imlerinden çıkar |
| `Enter` / `Boşluk` | Seçili haberi oku |
| `/` | Başlıklarda arama kutusuna odaklan |
| `Esc` | Aramayı temizle / Modaldan çık / Pencereyi gizle |
| `+` / `=` | Yazı tipi boyutunu büyüt |
| `-` / `_` | Yazı tipi boyutunu küçült |
| `0` | Yazı tipi boyutunu sıfırla |
| `R` | Akışları senkronize et |
| `?` | Bilgi ve Hakkında penceresini aç |

---

## Kurulum

### Gereksinimler
- Linux (Wayland önerilir, X11 desteklenir)
- Rust derleme araçları (1.75+)
- Quickshell (0.1.0+)
- `JetBrainsMono Nerd Font`

### Güvenli Yerel Kurulum
```bash
cd ~/Projects/omarchy/quicknews
./build.sh
```

Uygulamayı başlatmak için:
```bash
quicknews
```

---

## Güvenlik ve Bütünlük

QuickNews, HANCORE Linux standartlarına tam uyumlu olarak inşa edilmiştir:
- **Sıfır Emoji Politikası:** Tüm kaynak kodlarda, dokümantasyonlarda ve arayüzde emoji kullanılmaz.
- **Süreç İzolasyonu:** Arka plan görevleri yalıtılmış süreç gruplarında (`process_group(0)`) çalıştırılır.
- **Katı Dosya İzinleri:** Yapılandırma ve önbellek dosyaları yalnızca sahibine açık (`0600`) tutulur.
- **Bellek ve Veri Sınırları:** Ağ operasyonlarında 8 MiB tavan sınır uygulanır.

---

## Destek ve Sponsorluk

QuickNews gelişimini desteklemek isterseniz Buy Me a Coffee üzerinden katkıda bulunabilirsiniz:

<a href="https://buymeacoffee.com/ozdil" target="_blank">
  <img src="https://cdn.buymeacoffee.com/buttons/v2/default-yellow.png" alt="Buy Me A Coffee" width="180">
</a>

---

## Lisans

Bu proje MIT Lisansı altında lisanslanmıştır. Detaylar için [LICENSE](LICENSE) dosyasına bakabilirsiniz.
