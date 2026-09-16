# QuickNews

Guvenli, reklamsiz, resimsiz ve yapay zeka destekli minimalist haber okuyucu.

QuickNews; okuyucunun dikkatini dagitacak her turlu gorsel kirlilikten, reklamlardan, izleyicilerden ve sansasyonel basliklardan arindirilmis, saf tipografi ve metin odakli bir haber okuma deneyimi sunar.

Proje ilk olarak Linux ve Omarchy masaustu ortamina (Quickshell arayuzu ile) tam uyumlu olarak tasarlanmistir; cekirdek is mantigi (Rust) bagimsiz bir motor olarak gelistirildigi icin Windows ve Android platformlarina da kolaylikla genisletilebilir.

---

## Ozet Nitelikler

- **Tamamen Metin Odakli ve Resimsiz:** Makalelerdeki tum banner, gorsel (`<img>`), video, iframe ve script ogeleri ayiklanir; sadece saf haber metni ve baslik hiyerarsisi sunulur.
- **Sifir Reklam ve Izleyici:** DOM seviyesinde reklam, cerez uyarisi, sponsorlu baglanti ve sosyal medya kutucugu temizligi yapilir; baglantilardaki takip parametreleri (`utm_*`, `fbclid`) soyulur.
- **Yapay Zeka ile Dogal Dilde Kaynak Ekleme:** RSS veya XML adresi arama zorunlulugu yoktur. Arayuzde veya komut satirinda ornegin `"Turkiye'deki en iyi teknoloji sayfalarindan 10 tanesini ekle"` yazildiginda siteler otomatik tespit edilir, RSS akislari dogrulanir ve listeye eklenir.
- **3 Maddede Hap Ozet ve Notr Baslik:** Uzun haberler yapay zeka tarafindan analiz edilir, clickbait basliklar gercekci ve tarafsiz bir dille yeniden yazilir ve 3 maddelik anahtar cikarim sunulur.
- **Omarchy Ekosistemi ile Canli Uyum:** Omarchy `colors.toml` dosyasini anlik izler; tema degistiginde arayuz renkleri canli guncellenir.
- **Standart Tipografi:** Tum arayuz ve okuma alaninda zorunlu standart olan `JetBrainsMono Nerd Font` kullanilir.
- **Sert Guvenlik Yapilandirmasi (HANCORE Standartlari):**
  - SSRF (Server-Side Request Forgery) Korumasi: Ozel ag bloklari (RFC 1918), yerel baglanti (RFC 3927) ve geri dongu (loopback / 127.0.0.1 / ::1) istekleri kesin olarak engellenir.
  - Boyut ve Bellek Sinirlari: HTTP yanitlari ve dosya okumalari 1-2 MiB tavan sinirla (`take(MAX + 1)`) kisitlanir.
  - Atomik Depolama: Yapilandirma ve durum dosyalari `.tmp_*` uzerinden 0600 dosya ve 0700 dizin izinleriyle yazilir. Sembolik baglar kesinlikle reddedilir.

---

## Mimari Dizin Yapisi

```text
blissful-hopper/
|-- Cargo.toml                      Rust paket tanimi ve bagimliliklar
|-- CONTRIBUTING.md                 Katki kurallari ve HANCORE guvenlik yonergeleri
|-- README.md                       Proje dokumantasyonu
|-- PKGBUILD                        Arch Linux / Omarchy paket tanimi
|-- quicknews                       Ana baslatici betik (IPC toggle ve CLI delegasyonu)
|-- quicknews.desktop               XDG masaustu baslaticisi
|-- Panel.qml                       Omarchy Top Bar panel butonu
|-- src/
|   |-- lib.rs                      Rust cekirdek kutuphane api
|   |-- main.rs                     quicknews-engine CLI ve servis motoru
|   `-- core/
|       |-- security.rs             SSRF korumasi, atomik dosya, boyut sinirlari
|       |-- feed.rs                 RSS/Atom ayristirici ve Feed Auto-Discovery
|       |-- extractor.rs            Reklamsiz, resimsiz saf metin makale cikarici
|       |-- adblock.rs              Reklam ve izleyici temizleme kurallari
|       |-- ai.rs                   Dogal dil kaynak bulma ve 3 maddede ozet
|       `-- storage.rs              0600/0700 atomik onbellek ve kaynak yonetimi
`-- qml/
    |-- shell.qml                   Quickshell ShellRoot, FloatingWindow ve IPC
    |-- MainWindow.qml              3 panelli minimalist ana pencere
    |-- components/
    |   |-- SourceSidebar.qml       Sol panel: Kaynak ve kategori listesi
    |   |-- HeadlineList.qml        Orta panel: Saf metin haber akisi
    |   |-- ArticleReader.qml       Sag panel: Resimsiz ve reklamsiz okuma alani
    |   `-- AddSourceModal.qml      Yapay zeka ile dogal dil kaynak ekleme penceresi
    `-- theme/
        `-- Theme.qml               Omarchy colors.toml ve JetBrainsMono font tanimi
```

---

## Komut Satiri (CLI) Kullanim Rehberi

`quicknews-engine` veya `./quicknews` araciligiyla arayuz acilmadan dogrudan terminalden calisilabilir:

```bash
# Kaynaklari senkronize etme
quicknews sync

# Haberleri listeleme
quicknews list

# Bir haberin saf, reklamsiz metnini terminalde okuma
quicknews read "<Haber Baglantisi>"

# Yapay zeka ile 3 maddede ozet ve notr baslik alma
quicknews summarize "<Haber Baglantisi>"

# Dogal dille yeni kaynaklar bulup ekleme
quicknews add-prompt "Turkiye'deki en iyi teknoloji sayfalarindan 10 tanesini ekle"

# Ekli kaynaklari listeleme
quicknews sources

# Uygulama durumunu JSON formatinda sorgulama
quicknews status
```

---

## Quickshell Masaustu Arayuzu

Arayuz baslatildiginda:
- Sol tarafta kategoriler (`Teknoloji`, `Linux`, `Bilim`, `Ekonomi`) ve kaynak listesi yer alir.
- Ortada okuma sureleri belirtilmis saf metin haber akisi bulunur.
- Sag tarafta genisligi gozu yormayacak sekilde optimize edilmis saf metin okuyucu yer alir; gorsel, reklam veya dikkat dagitici hicbir unsur bulunmaz.
- `A+` ve `A-` dugmeleriyle yazi boyutu aninda ayarlanabilir.
- `Yapay Zeka Ozeti` butonu ile haberin notr basligi ve 3 maddelik hap ozeti goruntulenebilir.
- `AI ile Kaynak Ekle` dugmesiyle dogal dilde yeni kaynaklar tanimlanabilir.
- `q` veya `Escape` tusu ile pencere gizlenebilir; Omarchy Top Bar uzerindeki gazete simgesine tiklanarak aninda acilabilir.
