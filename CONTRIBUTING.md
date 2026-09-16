# QuickNews Katkı ve Geliştirme Standartları

Bu proje Omarchy Linux standartlarına ve HANCORE güvenlik yönergelerine tam uyumlu olarak geliştirilmektedir.

## 1. Tipografi ve Varsayılan Font
- Tüm arayüz ve bağımsız masaüstü bileşenlerinde varsayılan yazı tipi: `JetBrainsMono Nerd Font`.
- Font ailesi zinciri: `"JetBrainsMono Nerd Font, JetBrains Mono, monospace"`.
- Hiçbir bileşende Inter, Noto Sans, Cantarell veya genel sans-serif fontlar varsayılan olarak tanımlanmayacaktır.

## 2. Sıfır Emoji Politikası
- Kaynak kodlarda, dokümantasyonlarda, markdown dosyalarında, commit mesajlarında, PR/issue içeriklerinde ve kullanıcı arayüzünde kesinlikle hiçbir unicode emoji kullanılmayacaktır.
- Simgeler için yalnızca Nerd Font glifleri veya açık standart vektörler tercih edilecektir.

## 3. Güvenlik ve Mimari Standartları (HANCORE / Linux)
- Harici Süreç İzolasyonu: Alt süreçler ayrı süreç gruplarında (`process_group(0)`) çalıştırılacak ve RAII ProcessGroupGuard ile yönetilecektir.
- Boyut ve Bellek Sınırları: Ağ istekleri ve dosya okumalarında mutlaka boyut sınırı (1-2 MiB tavan sınır) uygulanacaktır (`take(MAX + 1)`).
- SSRF Savunması: Dış ağ isteklerinde özel ağ blokları (RFC 1918), yerel bağlantı (RFC 3927) ve geri döngü (loopback / 127.0.0.1 / ::1) adreslerine yapılan çağrılar engellenecektir.
- Atomik ve Güvenli Depolama: Kullanıcı yapılandırmaları ve önbellek dosyaları atomik `.tmp_*` dosyalarıyla, dosya izinleri 0600 ve dizin izinleri 0700 olarak yazılacaktır. Sembolik bağlar (`symlink_metadata`) kesinlikle reddedilecektir.
- Argüman ve Komut Enjeksiyonu Savunması: Harici komut çağrılarında ayrık argüman dizisi kullanılacak ve katı doğrulamadan geçirilecektir.
