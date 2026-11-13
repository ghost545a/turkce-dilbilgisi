//! Frequency dictionary: the most common Turkish verbs and adjectives.
//!
//! Entries are listed in rank order (most common first). The ordering is an
//! approximation drawn from general knowledge of Turkish frequency lists, not
//! a copy of any single corpus. To add a word, append a tuple to the list.

use serde::Serialize;

use crate::{fuzzy, normalize};

/// (infinitive, English, aorist 3sg, present continuous 3sg, governed case)
pub const VERBS: &[(&str, &str, &str, &str, &str)] = &[
    ("olmak", "to be, become", "olur", "oluyor", ""),
    ("etmek", "to do, make (auxiliary)", "eder", "ediyor", ""),
    ("yapmak", "to do, make", "yapar", "yapıyor", ""),
    ("demek", "to say", "der", "diyor", ""),
    ("gelmek", "to come", "gelir", "geliyor", "-(y)A"),
    ("gitmek", "to go", "gider", "gidiyor", "-(y)A"),
    ("almak", "to take, buy", "alır", "alıyor", ""),
    ("vermek", "to give", "verir", "veriyor", "-(y)A"),
    ("görmek", "to see", "görür", "görüyor", "-(y)I"),
    ("bilmek", "to know", "bilir", "biliyor", ""),
    ("istemek", "to want", "ister", "istiyor", ""),
    ("bulmak", "to find", "bulur", "buluyor", ""),
    ("kalmak", "to stay, remain", "kalır", "kalıyor", "-DA"),
    ("çıkmak", "to go out, go up", "çıkar", "çıkıyor", "-DAn / -(y)A"),
    ("bakmak", "to look", "bakar", "bakıyor", "-(y)A"),
    ("söylemek", "to say, tell", "söyler", "söylüyor", ""),
    ("çalışmak", "to work, try", "çalışır", "çalışıyor", ""),
    ("başlamak", "to begin", "başlar", "başlıyor", "-(y)A"),
    ("geçmek", "to pass", "geçer", "geçiyor", "-DAn"),
    ("girmek", "to enter", "girer", "giriyor", "-(y)A"),
    ("düşünmek", "to think", "düşünür", "düşünüyor", ""),
    ("konuşmak", "to speak, talk", "konuşur", "konuşuyor", "-(y)lA"),
    ("yaşamak", "to live", "yaşar", "yaşıyor", ""),
    ("durmak", "to stop, stand", "durur", "duruyor", ""),
    ("anlamak", "to understand", "anlar", "anlıyor", ""),
    ("sevmek", "to love, like", "sever", "seviyor", "-(y)I"),
    ("koymak", "to put", "koyar", "koyuyor", ""),
    ("getirmek", "to bring", "getirir", "getiriyor", ""),
    ("kullanmak", "to use", "kullanır", "kullanıyor", ""),
    ("tutmak", "to hold", "tutar", "tutuyor", ""),
    ("beklemek", "to wait", "bekler", "bekliyor", "-(y)I"),
    ("sormak", "to ask", "sorar", "soruyor", "-(y)A"),
    ("duymak", "to hear", "duyar", "duyuyor", ""),
    ("dönmek", "to turn, return", "döner", "dönüyor", "-(y)A"),
    ("okumak", "to read, study", "okur", "okuyor", ""),
    ("yazmak", "to write", "yazar", "yazıyor", ""),
    ("açmak", "to open", "açar", "açıyor", ""),
    ("oturmak", "to sit, reside", "oturur", "oturuyor", "-(y)A / -DA"),
    ("atmak", "to throw", "atar", "atıyor", ""),
    ("düşmek", "to fall", "düşer", "düşüyor", ""),
    ("yemek", "to eat", "yer", "yiyor", ""),
    ("içmek", "to drink", "içer", "içiyor", ""),
    ("göstermek", "to show", "gösterir", "gösteriyor", "-(y)A"),
    ("anlatmak", "to tell, explain", "anlatır", "anlatıyor", ""),
    ("bırakmak", "to leave, let go", "bırakır", "bırakıyor", ""),
    ("taşımak", "to carry", "taşır", "taşıyor", ""),
    ("kazanmak", "to win, earn", "kazanır", "kazanıyor", ""),
    ("ölmek", "to die", "ölür", "ölüyor", ""),
    ("varmak", "to arrive", "varır", "varıyor", "-(y)A"),
    ("sanmak", "to suppose", "sanır", "sanıyor", ""),
    ("dinlemek", "to listen", "dinler", "dinliyor", "-(y)I"),
    ("izlemek", "to watch, follow", "izler", "izliyor", ""),
    ("öğrenmek", "to learn", "öğrenir", "öğreniyor", ""),
    ("unutmak", "to forget", "unutur", "unutuyor", ""),
    ("hatırlamak", "to remember", "hatırlar", "hatırlıyor", ""),
    ("yürümek", "to walk", "yürür", "yürüyor", ""),
    ("koşmak", "to run", "koşar", "koşuyor", ""),
    ("uyumak", "to sleep", "uyur", "uyuyor", ""),
    ("kalkmak", "to get up", "kalkar", "kalkıyor", ""),
    ("satmak", "to sell", "satar", "satıyor", ""),
    ("ödemek", "to pay", "öder", "ödüyor", ""),
    ("seçmek", "to choose", "seçer", "seçiyor", ""),
    ("değişmek", "to change (by itself)", "değişir", "değişiyor", ""),
    ("değiştirmek", "to change (something)", "değiştirir", "değiştiriyor", ""),
    ("göndermek", "to send", "gönderir", "gönderiyor", ""),
    ("aramak", "to look for, call", "arar", "arıyor", "-(y)I"),
    ("kapatmak", "to close, turn off", "kapatır", "kapatıyor", ""),
    ("bitmek", "to end, run out", "biter", "bitiyor", ""),
    ("bitirmek", "to finish (something)", "bitirir", "bitiriyor", ""),
    ("hazırlamak", "to prepare", "hazırlar", "hazırlıyor", ""),
    ("oynamak", "to play", "oynar", "oynuyor", ""),
    ("gülmek", "to laugh", "güler", "gülüyor", "-(y)A"),
    ("ağlamak", "to cry", "ağlar", "ağlıyor", ""),
    ("kesmek", "to cut", "keser", "kesiyor", ""),
    ("çekmek", "to pull", "çeker", "çekiyor", ""),
    ("itmek", "to push", "iter", "itiyor", ""),
    ("vurmak", "to hit", "vurur", "vuruyor", "-(y)A"),
    ("kırmak", "to break", "kırar", "kırıyor", ""),
    ("taşınmak", "to move house", "taşınır", "taşınıyor", "-(y)A"),
    ("binmek", "to get on, ride", "biner", "biniyor", "-(y)A"),
    ("inmek", "to get off, go down", "iner", "iniyor", "-DAn"),
    ("yardım etmek", "to help", "yardım eder", "yardım ediyor", "-(y)A"),
    ("teşekkür etmek", "to thank", "teşekkür eder", "teşekkür ediyor", "-(y)A"),
    ("korkmak", "to be afraid", "korkar", "korkuyor", "-DAn"),
    ("inanmak", "to believe", "inanır", "inanıyor", "-(y)A"),
    ("güvenmek", "to trust", "güvenir", "güveniyor", "-(y)A"),
    ("özlemek", "to miss", "özler", "özlüyor", "-(y)I"),
    ("hissetmek", "to feel", "hisseder", "hissediyor", ""),
    ("karar vermek", "to decide", "karar verir", "karar veriyor", "-(y)A"),
    ("kabul etmek", "to accept", "kabul eder", "kabul ediyor", ""),
    ("devam etmek", "to continue", "devam eder", "devam ediyor", "-(y)A"),
    ("tanımak", "to know (a person), recognise", "tanır", "tanıyor", ""),
    ("tanışmak", "to get acquainted", "tanışır", "tanışıyor", "-(y)lA"),
    ("buluşmak", "to meet up", "buluşur", "buluşuyor", "-(y)lA"),
    ("evlenmek", "to get married", "evlenir", "evleniyor", "-(y)lA"),
    ("doğmak", "to be born", "doğar", "doğuyor", ""),
    ("büyümek", "to grow", "büyür", "büyüyor", ""),
    ("yetişmek", "to catch, be in time for", "yetişir", "yetişiyor", "-(y)A"),
    ("kaçmak", "to escape", "kaçar", "kaçıyor", "-DAn"),
    ("saklamak", "to hide, keep", "saklar", "saklıyor", ""),
    ("kaybetmek", "to lose", "kaybeder", "kaybediyor", ""),
    ("denemek", "to try", "dener", "deniyor", ""),
    ("gerekmek", "to be necessary", "gerekir", "gerekiyor", ""),
    ("yetmek", "to be enough", "yeter", "yetiyor", "-(y)A"),
    ("benzemek", "to resemble", "benzer", "benziyor", "-(y)A"),
    ("uymak", "to fit, comply", "uyar", "uyuyor", "-(y)A"),
    ("katılmak", "to join, agree", "katılır", "katılıyor", "-(y)A"),
    ("ayrılmak", "to leave, separate", "ayrılır", "ayrılıyor", "-DAn"),
    ("hoşlanmak", "to like", "hoşlanır", "hoşlanıyor", "-DAn"),
    ("nefret etmek", "to hate", "nefret eder", "nefret ediyor", "-DAn"),
    ("bahsetmek", "to mention", "bahseder", "bahsediyor", "-DAn"),
    ("sürmek", "to drive, last", "sürer", "sürüyor", ""),
    ("uçmak", "to fly", "uçar", "uçuyor", ""),
    ("yüzmek", "to swim", "yüzer", "yüzüyor", ""),
    ("pişirmek", "to cook", "pişirir", "pişiriyor", ""),
    ("yıkamak", "to wash", "yıkar", "yıkıyor", ""),
    ("temizlemek", "to clean", "temizler", "temizliyor", ""),
    ("giymek", "to wear, put on", "giyer", "giyiyor", ""),
    ("çıkarmak", "to take off, remove", "çıkarır", "çıkarıyor", ""),
    ("dinlenmek", "to rest", "dinlenir", "dinleniyor", ""),
    ("yorulmak", "to get tired", "yorulur", "yoruluyor", ""),
    ("uyanmak", "to wake up", "uyanır", "uyanıyor", ""),
    ("hastalanmak", "to fall ill", "hastalanır", "hastalanıyor", ""),
    ("iyileşmek", "to recover", "iyileşir", "iyileşiyor", ""),
    ("üzülmek", "to be sad", "üzülür", "üzülüyor", "-(y)A"),
    ("sevinmek", "to be glad", "sevinir", "seviniyor", "-(y)A"),
    ("kızmak", "to get angry", "kızar", "kızıyor", "-(y)A"),
    ("şaşırmak", "to be surprised", "şaşırır", "şaşırıyor", "-(y)A"),
    ("merak etmek", "to wonder, worry", "merak eder", "merak ediyor", "-(y)I"),
    ("ummak", "to hope", "umar", "umuyor", ""),
    ("dilemek", "to wish", "diler", "diliyor", ""),
    ("söz vermek", "to promise", "söz verir", "söz veriyor", "-(y)A"),
    ("yalan söylemek", "to lie", "yalan söyler", "yalan söylüyor", ""),
    ("sürdürmek", "to carry on (something)", "sürdürür", "sürdürüyor", ""),
    ("artmak", "to increase", "artar", "artıyor", ""),
    ("azalmak", "to decrease", "azalır", "azalıyor", ""),
    ("gelişmek", "to develop", "gelişir", "gelişiyor", ""),
    ("oluşmak", "to form, consist", "oluşur", "oluşuyor", "-DAn"),
    ("sağlamak", "to provide, ensure", "sağlar", "sağlıyor", ""),
    ("belirtmek", "to state", "belirtir", "belirtiyor", ""),
    ("açıklamak", "to explain, announce", "açıklar", "açıklıyor", ""),
    ("eklemek", "to add", "ekler", "ekliyor", "-(y)A"),
    ("saymak", "to count", "sayar", "sayıyor", ""),
    ("ölçmek", "to measure", "ölçer", "ölçüyor", ""),
    ("karşılaşmak", "to run into, encounter", "karşılaşır", "karşılaşıyor", "-(y)lA"),
    ("kurmak", "to set up, found", "kurar", "kuruyor", ""),
    ("yıkmak", "to demolish", "yıkar", "yıkıyor", ""),
    ("korumak", "to protect", "korur", "koruyor", ""),
    ("kurtarmak", "to rescue", "kurtarır", "kurtarıyor", ""),
    ("kurtulmak", "to be saved, get rid of", "kurtulur", "kurtuluyor", "-DAn"),
    ("dokunmak", "to touch", "dokunur", "dokunuyor", "-(y)A"),
    ("sarılmak", "to hug", "sarılır", "sarılıyor", "-(y)A"),
    ("öpmek", "to kiss", "öper", "öpüyor", ""),
    ("bağırmak", "to shout", "bağırır", "bağırıyor", "-(y)A"),
    ("susmak", "to be quiet", "susar", "susuyor", ""),
    ("gezmek", "to stroll, tour", "gezer", "geziyor", ""),
    ("ziyaret etmek", "to visit", "ziyaret eder", "ziyaret ediyor", "-(y)I"),
    ("davet etmek", "to invite", "davet eder", "davet ediyor", ""),
    ("uğramak", "to drop by", "uğrar", "uğruyor", "-(y)A"),
    ("ulaşmak", "to reach", "ulaşır", "ulaşıyor", "-(y)A"),
    ("yollamak", "to send off", "yollar", "yolluyor", ""),
    ("harcamak", "to spend", "harcar", "harcıyor", ""),
    ("biriktirmek", "to save up, collect", "biriktirir", "biriktiriyor", ""),
    ("kiralamak", "to rent", "kiralar", "kiralıyor", ""),
    ("doldurmak", "to fill", "doldurur", "dolduruyor", ""),
    ("boşaltmak", "to empty", "boşaltır", "boşaltıyor", ""),
    ("yakmak", "to burn, switch on", "yakar", "yakıyor", ""),
    ("söndürmek", "to put out, switch off", "söndürür", "söndürüyor", ""),
    ("asmak", "to hang", "asar", "asıyor", ""),
    ("dikmek", "to sew, plant", "diker", "dikiyor", ""),
    ("çizmek", "to draw", "çizer", "çiziyor", ""),
    ("boyamak", "to paint", "boyar", "boyuyor", ""),
    ("tamir etmek", "to repair", "tamir eder", "tamir ediyor", ""),
    ("bozmak", "to break, spoil", "bozar", "bozuyor", ""),
    ("bozulmak", "to break down", "bozulur", "bozuluyor", ""),
    ("düzeltmek", "to correct", "düzeltir", "düzeltiyor", ""),
    ("kontrol etmek", "to check", "kontrol eder", "kontrol ediyor", ""),
    ("tekrar etmek", "to repeat", "tekrar eder", "tekrar ediyor", ""),
    ("çevirmek", "to turn, translate", "çevirir", "çeviriyor", ""),
    ("basmak", "to press, step on", "basar", "basıyor", "-(y)A"),
    ("çalmak", "to play (an instrument), ring, steal", "çalar", "çalıyor", ""),
    ("dans etmek", "to dance", "dans eder", "dans ediyor", ""),
    ("şarkı söylemek", "to sing", "şarkı söyler", "şarkı söylüyor", ""),
    ("gülümsemek", "to smile", "gülümser", "gülümsüyor", ""),
    ("tartışmak", "to argue, discuss", "tartışır", "tartışıyor", "-(y)lA"),
    ("kavga etmek", "to fight, quarrel", "kavga eder", "kavga ediyor", "-(y)lA"),
    ("affetmek", "to forgive", "affeder", "affediyor", ""),
    ("özür dilemek", "to apologise", "özür diler", "özür diliyor", "-DAn"),
    ("şikâyet etmek", "to complain", "şikâyet eder", "şikâyet ediyor", "-DAn"),
    ("tercih etmek", "to prefer", "tercih eder", "tercih ediyor", "-(y)I"),
    ("önermek", "to suggest", "önerir", "öneriyor", ""),
    ("planlamak", "to plan", "planlar", "planlıyor", ""),
    ("başarmak", "to succeed, manage", "başarır", "başarıyor", ""),
    ("yenmek", "to defeat", "yener", "yeniyor", ""),
    ("kaybolmak", "to get lost", "kaybolur", "kayboluyor", ""),
    ("yaklaşmak", "to approach", "yaklaşır", "yaklaşıyor", "-(y)A"),
    ("uzaklaşmak", "to move away", "uzaklaşır", "uzaklaşıyor", "-DAn"),
    ("acıkmak", "to get hungry", "acıkır", "acıkıyor", ""),
    ("susamak", "to get thirsty", "susar", "susuyor", ""),
    ("gerçekleşmek", "to come true, take place", "gerçekleşir", "gerçekleşiyor", ""),
];

/// (adjective, English, antonym)
pub const ADJECTIVES: &[(&str, &str, &str)] = &[
    ("iyi", "good", "kötü"),
    ("büyük", "big", "küçük"),
    ("yeni", "new", "eski"),
    ("güzel", "beautiful, nice", "çirkin"),
    ("küçük", "small", "büyük"),
    ("çok", "many, much", "az"),
    ("kötü", "bad", "iyi"),
    ("eski", "old (of things)", "yeni"),
    ("uzun", "long, tall", "kısa"),
    ("önemli", "important", "önemsiz"),
    ("az", "few, little", "çok"),
    ("kolay", "easy", "zor"),
    ("zor", "difficult", "kolay"),
    ("genç", "young", "yaşlı"),
    ("yaşlı", "old (of people)", "genç"),
    ("kısa", "short", "uzun"),
    ("doğru", "correct, true", "yanlış"),
    ("yanlış", "wrong", "doğru"),
    ("farklı", "different", "aynı"),
    ("aynı", "same", "farklı"),
    ("yakın", "near", "uzak"),
    ("uzak", "far", "yakın"),
    ("sıcak", "hot", "soğuk"),
    ("soğuk", "cold", "sıcak"),
    ("mutlu", "happy", "mutsuz"),
    ("hızlı", "fast", "yavaş"),
    ("yavaş", "slow", "hızlı"),
    ("açık", "open; light (of colour)", "kapalı"),
    ("kapalı", "closed", "açık"),
    ("pahalı", "expensive", "ucuz"),
    ("ucuz", "cheap", "pahalı"),
    ("yüksek", "high", "alçak"),
    ("alçak", "low", "yüksek"),
    ("güçlü", "strong", "zayıf"),
    ("zayıf", "weak, thin", "güçlü"),
    ("zengin", "rich", "fakir"),
    ("fakir", "poor", "zengin"),
    ("temiz", "clean", "kirli"),
    ("kirli", "dirty", "temiz"),
    ("dolu", "full", "boş"),
    ("boş", "empty, free", "dolu"),
    ("ağır", "heavy", "hafif"),
    ("hafif", "light (of weight)", "ağır"),
    ("geniş", "wide", "dar"),
    ("dar", "narrow", "geniş"),
    ("kalın", "thick", "ince"),
    ("ince", "thin, fine", "kalın"),
    ("derin", "deep", "sığ"),
    ("erken", "early", "geç"),
    ("geç", "late", "erken"),
    ("hazır", "ready", ""),
    ("meşgul", "busy", "müsait"),
    ("yorgun", "tired", "dinç"),
    ("hasta", "ill", "sağlıklı"),
    ("sağlıklı", "healthy", "hasta"),
    ("aç", "hungry", "tok"),
    ("tok", "full (not hungry)", "aç"),
    ("güvenli", "safe", "tehlikeli"),
    ("tehlikeli", "dangerous", "güvenli"),
    ("sessiz", "quiet", "gürültülü"),
    ("gürültülü", "noisy", "sessiz"),
    ("kalabalık", "crowded", "tenha"),
    ("rahat", "comfortable", "rahatsız"),
    ("serbest", "free, unrestricted", ""),
    ("özel", "special, private", "genel"),
    ("genel", "general", "özel"),
    ("basit", "simple", "karmaşık"),
    ("karmaşık", "complex", "basit"),
    ("mümkün", "possible", "imkânsız"),
    ("imkânsız", "impossible", "mümkün"),
    ("gerekli", "necessary", "gereksiz"),
    ("faydalı", "useful", "zararlı"),
    ("zararlı", "harmful", "faydalı"),
    ("ilginç", "interesting", "sıkıcı"),
    ("sıkıcı", "boring", "ilginç"),
    ("komik", "funny", ""),
    ("ciddi", "serious", ""),
    ("akıllı", "clever", "aptal"),
    ("aptal", "stupid", "akıllı"),
    ("çalışkan", "hard-working", "tembel"),
    ("tembel", "lazy", "çalışkan"),
    ("nazik", "polite, kind", "kaba"),
    ("kaba", "rude, rough", "nazik"),
    ("dürüst", "honest", ""),
    ("cesur", "brave", "korkak"),
    ("korkak", "cowardly", "cesur"),
    ("sakin", "calm", "sinirli"),
    ("sinirli", "irritable, nervous", "sakin"),
    ("üzgün", "sad", "mutlu"),
    ("şanslı", "lucky", "şanssız"),
    ("ünlü", "famous", ""),
    ("yabancı", "foreign", "yerli"),
    ("yerli", "local, domestic", "yabancı"),
    ("modern", "modern", ""),
    ("taze", "fresh", "bayat"),
    ("bayat", "stale", "taze"),
    ("tatlı", "sweet", "acı"),
    ("acı", "bitter, spicy", "tatlı"),
    ("tuzlu", "salty", "tuzsuz"),
    ("ekşi", "sour", ""),
    ("lezzetli", "delicious", "lezzetsiz"),
    ("sert", "hard, harsh", "yumuşak"),
    ("yumuşak", "soft", "sert"),
    ("kuru", "dry", "ıslak"),
    ("ıslak", "wet", "kuru"),
    ("parlak", "bright, shiny", "mat"),
    ("karanlık", "dark", "aydınlık"),
    ("aydınlık", "bright, well lit", "karanlık"),
    ("beyaz", "white", "siyah"),
    ("siyah", "black", "beyaz"),
    ("kırmızı", "red", ""),
    ("mavi", "blue", ""),
    ("yeşil", "green", ""),
    ("sarı", "yellow", ""),
    ("turuncu", "orange", ""),
    ("mor", "purple", ""),
    ("pembe", "pink", ""),
    ("kahverengi", "brown", ""),
    ("gri", "grey", ""),
    ("düz", "flat, straight", "eğri"),
    ("yuvarlak", "round", ""),
    ("keskin", "sharp", "kör"),
    ("sağlam", "sturdy, sound", "çürük"),
    ("kırık", "broken", "sağlam"),
    ("bozuk", "out of order", ""),
    ("doğal", "natural", "yapay"),
    ("yapay", "artificial", "doğal"),
    ("resmî", "official, formal", ""),
    ("gizli", "secret", "açık"),
    ("belli", "obvious, certain", "belirsiz"),
    ("emin", "sure", ""),
    ("haklı", "right, justified", "haksız"),
    ("suçlu", "guilty", "masum"),
    ("masum", "innocent", "suçlu"),
    ("bekâr", "single, unmarried", "evli"),
    ("evli", "married", "bekâr"),
    ("yalnız", "alone, lonely", ""),
    ("kızgın", "angry", ""),
    ("heyecanlı", "excited, exciting", ""),
    ("şaşkın", "surprised, bewildered", ""),
    ("gururlu", "proud", ""),
    ("utangaç", "shy", ""),
    ("sabırlı", "patient", "sabırsız"),
    ("dikkatli", "careful", "dikkatsiz"),
    ("başarılı", "successful", "başarısız"),
    ("mükemmel", "perfect", ""),
    ("harika", "wonderful", "berbat"),
    ("berbat", "awful", "harika"),
    ("normal", "normal", ""),
    ("son", "last, final", "ilk"),
];

#[derive(Serialize)]
struct Item<'a> {
    rank: usize,
    tr: &'a str,
    en: &'a str,
    aorist: &'a str,
    cont: &'a str,
    gov: &'a str,
    antonym: &'a str,
}

#[derive(Serialize)]
struct Results<'a> {
    kind: &'a str,
    total: usize,
    items: Vec<Item<'a>>,
}

fn items(kind: &str) -> Vec<Item<'static>> {
    if kind == "adjectives" {
        ADJECTIVES
            .iter()
            .enumerate()
            .map(|(i, &(tr, en, antonym))| Item { rank: i + 1, tr, en, aorist: "", cont: "", gov: "", antonym })
            .collect()
    } else {
        VERBS
            .iter()
            .enumerate()
            .map(|(i, &(tr, en, aorist, cont, gov))| Item { rank: i + 1, tr, en, aorist, cont, gov, antonym: "" })
            .collect()
    }
}

/// Entries of `kind` ("verbs" or "adjectives") matching `query`, most common
/// first, as JSON. An empty query returns the whole list. Matching is
/// diacritic-insensitive and covers the Turkish word, its forms and the
/// English meaning.
pub fn search(kind: &str, query: &str) -> String {
    let q = normalize(query.trim());
    let all = items(kind);
    let total = all.len();
    let items: Vec<Item> = all
        .into_iter()
        .filter(|it| {
            q.is_empty()
                || fuzzy(&normalize(&format!("{} {} {} {} {}", it.tr, it.en, it.aorist, it.cont, it.antonym)), &q)
        })
        .collect();
    serde_json::to_string(&Results { kind, total, items }).unwrap_or_else(|_| "null".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn sizes_and_uniqueness() {
        assert_eq!(VERBS.len(), 200);
        assert_eq!(ADJECTIVES.len(), 150);
        let verbs: HashSet<_> = VERBS.iter().map(|v| v.0).collect();
        assert_eq!(verbs.len(), VERBS.len(), "duplicate verb");
        let adjectives: HashSet<_> = ADJECTIVES.iter().map(|a| a.0).collect();
        assert_eq!(adjectives.len(), ADJECTIVES.len(), "duplicate adjective");
    }

    #[test]
    fn verbs_are_well_formed() {
        for &(inf, en, aorist, cont, gov) in VERBS {
            assert!(inf.ends_with("mak") || inf.ends_with("mek"), "{inf}");
            assert!(en.starts_with("to "), "{inf}: {en}");
            assert!(aorist.ends_with('r'), "{inf}: {aorist}");
            assert!(cont.ends_with("yor"), "{inf}: {cont}");
            for g in gov.split(" / ").filter(|g| !g.is_empty()) {
                assert!(["-(y)A", "-DA", "-DAn", "-(y)I", "-(y)lA"].contains(&g), "{inf}: {g}");
            }
        }
    }

    #[test]
    fn search_matches_turkish_and_english() {
        let all = search("verbs", "");
        assert!(all.contains("\"total\":200") && all.contains("\"rank\":200"));
        let go = search("verbs", "go");
        assert!(go.contains("\"gitmek\""));
        assert!(go.find("\"gitmek\"").unwrap() < go.find("\"çıkmak\"").unwrap(), "rank order kept");
        assert!(search("adjectives", "guzel").contains("\"güzel\""));
        assert!(search("adjectives", "expensive").contains("\"pahalı\""));
        assert!(search("verbs", "zzzz").contains("\"items\":[]"));
    }
}
