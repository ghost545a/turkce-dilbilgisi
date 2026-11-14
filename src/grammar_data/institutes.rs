//! Third batch of rules: topics taught in the textbook series of the official
//! institutes (İstanbul University's "İstanbul Yabancılar İçin Türkçe", Yunus
//! Emre Institute's "Yedi İklim", Ankara University TÖMER's "Yeni Hitit")
//! that the map did not yet cover. Runs after `extra::extend_grammar`, so it
//! can append to branches created there.

use super::{add, b, l, NodeDef};

pub(super) fn extend_institutes(root: &mut NodeDef) {
    add(root, &["Cümle Yapısı", "Bağlaçlar"], conjunctions());
    add(root, &["Cümle Yapısı", "Edatlar"], postpositions());
    add(root, &["İsimler & Çekim", "İsmin Hâlleri"], cases());
    add(root, &["Sıfatlar & Zarflar", "Küçültme & Benzerlik"], diminutives());
    add(root, &["Sıfatlar & Zarflar"], adjectives());
    add(root, &["Sıfatlar & Zarflar", "Zarflar"], adverbs());
    add(root, &["Sıfatlar & Zarflar", "İkilemeler"], repetition());
    add(root, &["Ek Fiil"], copula());
    add(root, &["Fiiller & Zamanlar", "Birleşik Zamanlar"], compound_tenses());
    add(root, &["Fiiller & Zamanlar", "Fiil Çatısı"], voice());
    add(root, &["Fiiller & Zamanlar", "Olumsuzluk & Yeterlilik"], negation());
    add(root, &["Fiiller & Zamanlar", "Görünüş Yapıları"], aspect());
    add(root, &["Fiilimsiler", "Zarf-Fiil"], converbs());
    add(root, &["Fiilimsiler", "Sıfat-Fiil"], participles());
    add(root, &["Fiilimsiler", "Zarf Cümlecikleri"], adverbial_clauses());
    add(root, &["Cümle Yapısı"], reported_speech());
    add(root, &["Cümle Yapısı", "Soru Türleri"], questions());
    add(root, &["Cümle Yapısı"], vec![meaning()]);
}

fn conjunctions() -> Vec<NodeDef> {
    vec![
        l("bu yüzden / bu nedenle", "so / therefore", "A2", "cause. Bu yüzden / bu nedenle / bu sebeple / onun için + result",
          "Invariable connectors that open the result sentence — the mirror image of çünkü, which opens the cause.",
          "bu yüzden is the most colloquial; bu nedenle and bu sebeple are neutral to formal; o yüzden and onun için are spoken.",
          [("Çok yorgunum, [bu yüzden] erken yatacağım.", "I am very tired, so I will go to bed early."),
           ("Yağmur yağıyordu, [bu nedenle] maç ertelendi.", "It was raining; therefore the match was postponed."),
           ("Param yoktu, [onun için] gelemedim.", "I had no money; that is why I couldn't come.")]),
        l("belki … belki / ister … ister", "maybe … maybe / whether … or", "A2", "belki X belki Y · ister X ister Y",
          "Invariable pairs. belki … belki weighs two possibilities; ister … ister (from iste-) says the choice makes no difference and is followed by an imperative or optative.",
          "Also gerek … gerek(se) ‘both … and’ (formal) and olsun … olsun ‘be it … or’.",
          [("[Belki] gelir, [belki] gelmez.", "Maybe he will come, maybe he won't."),
           ("[İster] gel, [ister] gelme.", "Come or don't come, as you wish."),
           ("[İster] çay [ister] kahve iç, fark etmez.", "Drink tea or coffee, it makes no difference.")]),
        l("sanki", "as if / as though", "B2", "sanki + clause (+ -mIş gibi)",
          "Invariable. Introduces an unreal comparison, usually reinforced by -mIş gibi or -(I)yormuş gibi at the end of the clause.",
          "In an exclamation it expresses protest: Sanki ben yaptım! ‘As if it were me who did it!’",
          [("[Sanki] beni tanımıyormuş gibi davrandı.", "He behaved as if he didn't know me."),
           ("[Sanki] dün gibi hatırlıyorum.", "I remember it as though it were yesterday."),
           ("[Sanki] sen hiç hata yapmıyorsun!", "As if you never made mistakes!")]),
        l("meğer / meğerse", "it turns out that", "B2", "meğer(se) + clause in -mIş / -(y)mIş",
          "Invariable. Introduces a fact discovered later that corrects an earlier belief; the verb takes the evidential.",
          "Not to be confused with meğerki ‘unless’.",
          [("Onu yabancı sanıyordum, [meğer] Türk'müş.", "I thought he was a foreigner; it turns out he is Turkish."),
           ("[Meğerse] her şeyi biliyormuş.", "It turns out he knew everything."),
           ("Kapıyı kilitledim sanmıştım, [meğer] açık kalmış.", "I thought I had locked the door; it turns out it was left open.")]),
        l("ne var ki / ne yazık ki / neyse ki", "however / unfortunately / fortunately", "B2", "clause. Ne var ki / ne yazık ki / neyse ki + clause",
          "Fixed connectors built on ki. ne var ki = ‘however’ (written); ne yazık ki = ‘unfortunately’; neyse ki = ‘luckily’.",
          "Synonyms: ancak, maalesef, iyi ki.",
          [("Çok çalıştı; [ne var ki] başaramadı.", "He worked hard; however, he did not succeed."),
           ("[Ne yazık ki] biletler tükenmiş.", "Unfortunately the tickets are sold out."),
           ("Geç kaldık; [neyse ki] film başlamamıştı.", "We were late; fortunately the film had not started.")]),
        l("üstelik / hatta / ayrıca", "moreover / even / besides", "B2", "clause, üstelik / hatta / ayrıca + clause",
          "Additive connectors. üstelik adds a stronger point; hatta goes one step further (‘even, indeed’); ayrıca adds a separate point.",
          "Often followed by de / da: üstelik de. Formal: buna ek olarak, bunun yanı sıra.",
          [("Ev çok küçük, [üstelik] pahalı.", "The house is very small and, what is more, expensive."),
           ("Onu tanıyorum, [hatta] çok iyi tanıyorum.", "I know him; indeed, I know him very well."),
           ("[Ayrıca] bir şey daha söylemek istiyorum.", "Besides, I want to say one more thing.")]),
        l("bununla birlikte", "nevertheless", "B2", "clause. Bununla birlikte / bununla beraber / buna rağmen + clause",
          "Concessive connectors that open a new sentence: ‘in spite of that’.",
          "yine de ‘still’ stands inside the clause. Formal: buna karşın.",
          [("Hava soğuktu; [bununla birlikte] yürüyüşe çıktık.", "The weather was cold; nevertheless we went for a walk."),
           ("Çok yorgundu, [buna rağmen] çalışmaya devam etti.", "He was very tired; despite that he carried on working."),
           ("Pahalı, [yine de] alacağım.", "It is expensive; still, I will buy it.")]),
        l("demek ki / yeter ki / tut ki", "so then / as long as / suppose", "C1", "demek ki + conclusion · yeter ki + optative or imperative · tut ki + clause",
          "Fixed combinations with the conjunction ki. demek ki draws a conclusion; yeter ki states the one condition that matters; tut ki (or diyelim ki) sets up a supposition.",
          "Written as two words. demek alone also means ‘so’.",
          [("Işıklar yanıyor, [demek ki] evdeler.", "The lights are on, so they must be at home."),
           ("Ne istersen yaparım, [yeter ki] sen mutlu ol.", "I will do whatever you want, as long as you are happy."),
           ("[Tut ki] kazandın, sonra ne yapacaksın?", "Suppose you won; what will you do then?")]),
        l("kaldı ki / şöyle ki / nasıl ki", "moreover / namely / just as", "C1", "kaldı ki + further argument · şöyle ki + explanation · nasıl ki … (öyle)",
          "Written-register connectors with ki. kaldı ki adds a clinching argument; şöyle ki introduces an explanation; nasıl ki sets up a comparison.",
          "Also illa ki ‘without fail’ and nedir ki ‘what is … after all’. All are written as two words.",
          [("Vaktim yok; [kaldı ki] param da yok.", "I have no time; and besides, I have no money either."),
           ("Durum [şöyle ki]: iki seçeneğimiz var.", "The situation is as follows: we have two options."),
           ("[Nasıl ki] sen beni dinledin, ben de seni dinleyeceğim.", "Just as you listened to me, I will listen to you.")]),
        l("aksi hâlde / açıkçası", "otherwise / frankly", "C1", "clause. Aksi hâlde / aksi takdirde + clause · Açıkçası + clause",
          "aksi hâlde and aksi takdirde are the formal equivalents of yoksa ‘otherwise’. açıkçası is a stance adverb: ‘to be frank’.",
          "Other stance adverbs: doğrusu, kısacası, anlaşılan, bana kalırsa.",
          [("Hemen ödeme yapın; [aksi hâlde] siparişiniz iptal edilir.", "Pay at once; otherwise your order will be cancelled."),
           ("[Açıkçası] bu fikri pek beğenmedim.", "Frankly, I did not much like this idea."),
           ("Dikkatli ol, [aksi takdirde] pişman olursun.", "Be careful, otherwise you will regret it.")]),
    ]
}

fn postpositions() -> Vec<NodeDef> {
    vec![
        l("-DAn … -(y)A kadar", "from … to / until", "A1", "N-ABL + N-DAT + kadar",
          "The starting point takes the ablative; the end point takes the dative followed by kadar. Used for both time and space.",
          "dek and değin are literary synonyms of kadar.",
          [("Dokuz[dan] beş[e kadar] çalışıyorum.", "I work from nine to five."),
           ("Ev[den] okul[a kadar] yürüdük.", "We walked from home to school."),
           ("Sabah[tan] akşam[a kadar] ders çalıştı.", "He studied from morning till evening.")]),
        l("nedeniyle / dolayısıyla", "owing to / consequently", "C1", "N-Ø nedeniyle / dolayısıyla · clause. Dolayısıyla + clause",
          "Formal causal postpositions built on neden and dolayı + possessive + instrumental. dolayısıyla is also a sentence connector: ‘consequently’.",
          "Synonyms: sebebiyle, yüzünden, -DAn dolayı. With a clause: -mAsI nedeniyle.",
          [("Kar [nedeniyle] okullar tatil edildi.", "Schools were closed owing to snow."),
           ("Uçuş, sis [dolayısıyla] ertelendi.", "The flight was postponed on account of fog."),
           ("Hastaydım, [dolayısıyla] toplantıya katılamadım.", "I was ill; consequently I could not attend the meeting.")]),
        l("dışında / yanı sıra", "apart from / as well as", "C1", "N-(n)In dışında · N-(n)In yanı sıra",
          "dışında = ‘except, outside of’; yanı sıra = ‘alongside, in addition to’. The genitive on the noun is often dropped.",
          "bunun dışında ‘apart from this’, bunun yanı sıra ‘in addition’. Compare -DAn başka.",
          [("Pazar [dışında] her gün açığız.", "We are open every day except Sunday."),
           ("İngilizcenin [yanı sıra] Almanca da biliyor.", "As well as English, she also knows German."),
           ("Bunun [dışında] bir sorun yok.", "Apart from this, there is no problem.")]),
    ]
}

fn cases() -> Vec<NodeDef> {
    vec![
        l("Durum Eklerinin Yan İşlevleri", "Secondary uses of the case suffixes", "C1", "-(y)A · -DA · -DAn · -(y)lA beyond place",
          "Besides direction, place and source, the cases express price and purpose (dative); time, age and state (locative); cause, material, part and comparison (ablative); manner and time (instrumental).",
          "beş liraya aldım; yirmi yaşında; korkudan titredi, tahtadan masa, öğrencilerden biri; dikkatle dinledi, zamanla alışırsın.",
          [("Bu kitabı on lira[ya] aldım.", "I bought this book for ten lira."),
           ("Soğuk[tan] titriyordu.", "He was shivering from the cold."),
           ("Zaman[la] her şeye alışırsın.", "In time you get used to everything.")]),
    ]
}

fn diminutives() -> Vec<NodeDef> {
    vec![
        l("-CAğIz", "diminutive of pity / affection", "A2", "-cağız / -ceğiz · -çağız / -çeğiz",
          "2-way harmony on A plus assimilation; the last vowel is only ı / i.",
          "Mostly on words for people and animals: çocukcağız, adamcağız, kadıncağız, hayvancağız, kızcağız. Expresses sympathy.",
          [("Çocuk[cağız] çok üşümüş.", "The poor child is very cold."),
           ("Adam[cağız] ne yapacağını şaşırdı.", "The poor man did not know what to do."),
           ("Hayvan[cağız] günlerdir aç.", "The poor animal has been hungry for days.")]),
    ]
}

fn adjectives() -> Vec<NodeDef> {
    vec![
        l("Sıfatların Sıralanışı", "Order of modifiers", "C1", "demonstrative + number + adjective + (bir) + noun",
          "Demonstratives come first, then numerals, then describing adjectives; indefinite bir stands directly before the noun. A relative clause precedes everything, or follows the demonstrative.",
          "şu iki büyük ev; güzel bir ev; bu benim aldığım kitap / benim aldığım bu kitap.",
          [("[Şu iki küçük] çocuk kardeş.", "Those two small children are siblings."),
           ("[Eski bir ahşap] evde oturuyorlar.", "They live in an old wooden house."),
           ("[Dün aldığım o kırmızı] elbiseyi giyeceğim.", "I will wear that red dress I bought yesterday.")]),
        l("Adlaşmış Sıfat", "Adjectives used as nouns", "C1", "Adj + plural / possessive / case",
          "An adjective standing without its noun takes the noun's suffixes. With the 3rd-person possessive it means ‘the … one (of them)’.",
          "gençler ‘the young’, zenginler ‘the rich’, büyüğü ‘the big one’, en iyisi ‘the best (thing)’, kırmızıyı ‘the red one’.",
          [("[Gençler] bu müziği seviyor.", "Young people like this music."),
           ("Bana [büyüğünü] ver.", "Give me the big one."),
           ("[En iyisi] yarın konuşmak.", "The best thing is to talk tomorrow.")]),
    ]
}

fn adverbs() -> Vec<NodeDef> {
    vec![
        l("zaten / şimdiden / şimdilik", "anyway, already / already now / for now", "B2", "zaten · şimdiden · şimdilik",
          "Invariable adverbs. zaten = ‘already, in any case’; şimdiden = ‘as early as now’; şimdilik = ‘for the time being’.",
          "Contrast henüz ‘yet, just’ (with a negative ‘not yet’) and artık ‘from now on / no longer’.",
          [("Ben [zaten] biliyordum.", "I already knew anyway."),
           ("[Şimdiden] teşekkür ederim.", "Thank you in advance."),
           ("[Şimdilik] bu kadar yeter.", "This is enough for now.")]),
        l("hiç değilse / hiç yoktan", "at least / out of nothing", "C1", "hiç değilse / hiç olmazsa + clause · hiç yoktan + clause",
          "Fixed adverbial phrases with hiç. hiç değilse and hiç olmazsa = ‘at the very least’ (also en azından); hiç yoktan = ‘for no reason, from nothing’.",
          "Hiç yoktan iyidir ‘better than nothing’.",
          [("[Hiç değilse] bir telefon edebilirdin.", "You could at least have phoned."),
           ("[Hiç yoktan] kavga çıkardı.", "He started a fight out of nothing."),
           ("[Hiç olmazsa] yarın gel.", "At least come tomorrow.")]),
    ]
}

fn repetition() -> Vec<NodeDef> {
    vec![
        l("X mI X", "Emphatic repetition with mI", "C1", "Adj + mI + Adj",
          "An adjective or adverb repeated around the particle mI: ‘really, ever so’. 4-way harmony on mI; no question is meant.",
          "güzel mi güzel, uzun mu uzun, zor mu zor. Colloquial and expressive.",
          [("Güzel [mi] güzel bir kız.", "A really, really beautiful girl."),
           ("Yol uzun [mu] uzun.", "The road is ever so long."),
           ("Akıllı [mı] akıllı bir çocuk.", "An exceedingly clever child.")]),
        l("-mAsInA … (ama)", "Concessive verb repetition", "C1", "V + ma + (s)ı + n + a  V + tense, ama …",
          "The verb is said first as -mAsInA and then again in a finite form: ‘as far as …-ing goes, yes — but’.",
          "Also with dA: Geldi gelmesine de… Always followed by a reservation.",
          [("Git[mesine] gittim ama onu göremedim.", "I did go, but I couldn't see him."),
           ("Güzel ol[masına] güzel, ama çok pahalı.", "It is beautiful all right, but very expensive."),
           ("Oku[masına] okudum, ama anlamadım.", "I did read it, but I didn't understand.")]),
    ]
}

fn copula() -> Vec<NodeDef> {
    vec![
        l("olmak ile İsim Cümleleri", "‘To be’ in other tenses (ol-)", "A2", "N / Adj + ol- + tense",
          "The suffixed copula exists only for the present, past, evidential and conditional. Every other tense or mood of ‘to be’ uses the verb ol-: olacak, olur, olmalı, olabilir, olsun.",
          "ol- also means ‘become’: doktor oldu ‘he became a doctor’. Negative: olmayacak.",
          [("Yarın evde [olacağım].", "I will be at home tomorrow."),
           ("Hava güzel [olursa] gideriz.", "If the weather is nice, we will go."),
           ("Daha dikkatli [olmalısın].", "You must be more careful.")]),
        l("-mIşlIğI var / -(y)AcAğI yok", "Existence patterns on verbs", "C1", "V + mIş + lIk + possessive + var / yok · V + (y)AcAk / DIk + possessive + var / yok / ol-",
          "A nominalised verb as the subject of var, yok or ol-. -mIşlIğI var = ‘has (at some time) done’; -(y)AcAğI yok = ‘shows no sign of doing’; -DIğI oldu = ‘there were times when’.",
          "Experience in the negative: Hiç gitmişliğim yok ‘I have never been’.",
          [("Oraya git[mişliğim var].", "I have been there at some point."),
           ("Yağmurun din[eceği yok].", "The rain shows no sign of stopping."),
           ("Onu kır[dığım oldu].", "There were times when I hurt him.")]),
    ]
}

fn compound_tenses() -> Vec<NodeDef> {
    vec![
        l("-(I)yordu", "Past continuous", "B1", "V + (I)yor + du + person",
          "Present continuous + past copula: an action in progress at a past moment, or a past habit.",
          "Negative: -mIyordu. Question: geliyor muydun? Persons: -m, -n, Ø, -k, -nuz, -lar.",
          [("Sen aradığında yemek yi[yordu]m.", "I was eating when you called."),
           ("O zamanlar Ankara'da otur[uyordu]k.", "In those days we were living in Ankara."),
           ("Ne yap[ıyordu]n?", "What were you doing?")]),
        l("-(A/I)rdI", "Past habitual / would", "B2", "V + aorist + dı / di / du / dü + person",
          "Aorist + past copula: ‘used to’, or the result clause of an unreal condition: ‘would (have)’.",
          "Negative: -mAzdI. Polite wish: isterdim ‘I would like’.",
          [("Çocukken her gün denize gir[erdi]k.", "As children we used to go swimming every day."),
           ("Vaktim olsa gel[irdi]m.", "If I had time, I would come."),
           ("Bir kahve iç[erdi]m.", "I would gladly have a coffee.")]),
        l("-(y)AcAktI", "Future in the past", "B2", "V + (y)acak / (y)ecek + tı / ti + person",
          "Future + past copula: something that was going to happen, often unrealised.",
          "With az kalsın or neredeyse: ‘very nearly’. Also the result of an unreal past condition.",
          [("Sana söyle[yecekti]m ama unuttum.", "I was going to tell you, but I forgot."),
           ("Dün gel[ecekti], gelmedi.", "He was going to come yesterday; he didn't."),
           ("Az kalsın düş[ecekti]m.", "I very nearly fell.")]),
        l("-mIştI", "Past perfect", "B2", "V + mış / miş / muş / müş + tı / ti / tu / tü + person",
          "Evidential past + past copula: an action completed before another past moment. The hearsay meaning disappears.",
          "In speech also for a remote past event: Geçen yıl gitmiştik.",
          [("Ben geldiğimde herkes git[mişti].", "When I arrived everyone had left."),
           ("Bu filmi daha önce gör[müştü]m.", "I had seen this film before."),
           ("Sana söyle[miştim]!", "I told you so!")]),
        l("-mAlIydI", "should have", "B1", "V + malı / meli + ydı / ydi + person",
          "Necessitative + past copula: an obligation in the past, usually one that was not met.",
          "Negative: -mAmAlIydI ‘should not have’.",
          [("Daha erken çık[malıydı]k.", "We should have left earlier."),
           ("Ona söyle[memeliydi]n.", "You should not have told him."),
           ("Doktora git[meliydi]m.", "I should have gone to the doctor.")]),
        l("-sAydI", "if only … had / had …", "B1", "V + sa / se + ydı / ydi + person",
          "Conditional + past copula: an unreal condition or wish about the past. The result clause takes -(A/I)rdI.",
          "Keşke + -sAydI = ‘if only’. The separate form (gelse idi) is archaic.",
          [("Keşke daha çok çalış[saydı]m.", "If only I had studied more."),
           ("Bil[seydi]m gelirdim.", "Had I known, I would have come."),
           ("Erken kalk[saydı]n yetişirdin.", "If you had got up early, you would have made it.")]),
        l("-(I)yormuş / -(A/I)rmIş", "Reported present and habit", "B2", "V + (I)yor / aorist + muş / mış + person",
          "Present or aorist + evidential copula: the speaker reports what they were told, or has just found out.",
          "Also used in fairy tales and when retelling dreams. Negative: -mIyormuş, -mAzmIş.",
          [("Ali yeni bir iş arı[yormuş].", "Apparently Ali is looking for a new job."),
           ("Eskiden burada bir çınar ağacı dur[urmuş].", "They say a plane tree used to stand here."),
           ("Sen beni arı[yormuşsun].", "I hear you have been looking for me.")]),
        l("-(y)AcAkmIş", "Reported future", "B2", "V + (y)acak / (y)ecek + mış / miş + person",
          "Future + evidential copula: a plan or prediction the speaker has only heard about.",
          "Can convey doubt or irony, especially with güya ‘supposedly’.",
          [("Yarın kar yağ[acakmış].", "It is supposedly going to snow tomorrow."),
           ("Toplantı ertelen[ecekmiş].", "I hear the meeting is going to be postponed."),
           ("Güya bana yardım ed[ecekmiş].", "Supposedly he was going to help me.")]),
        l("-mIşmIş / -mAlIymIş / -sAymIş", "Reported past, obligation and condition", "C1", "V + mIş + mIş · V + mAlI + ymIş · V + sA + ymIş",
          "The evidential copula on the evidential past (often sceptical: ‘allegedly’), on the necessitative (‘apparently one must’) and on the conditional (‘if, as they say’).",
          "-mIşmIş is colloquial and mocking: Çok çalışmışmış!",
          [("Güya çok çalış[mışmış].", "Allegedly he worked very hard."),
           ("Doktora göre daha çok yürü[meliymiş]im.", "According to the doctor I ought to walk more."),
           ("İste[seymiş] yapabilirmiş.", "Apparently he could have done it if he had wanted.")]),
        l("-(A/I)rsA", "Real condition (aorist)", "B1", "V + aorist + sa / se + person",
          "Aorist + conditional copula: a general or future condition. The result takes the aorist, the future or an imperative.",
          "Negative: -mAzsA. The most common ‘if’ form.",
          [("Erken gel[irse]n birlikte yemek yeriz.", "If you come early, we will eat together."),
           ("Yağmur yağ[arsa] şemsiyeni al.", "If it rains, take your umbrella."),
           ("Çalış[mazsa]n başaramazsın.", "If you don't work, you won't succeed.")]),
        l("-(I)yorsA", "Condition on an ongoing action", "B2", "V + (I)yor + sa + person",
          "Present continuous + conditional copula: ‘if (it is true that) … is doing’.",
          "Negative: -mIyorsA.",
          [("Üşü[yorsa]n pencereyi kapat.", "If you are cold, close the window."),
           ("Anlamı[yorsa]nız tekrar anlatayım.", "If you don't understand, let me explain again."),
           ("Uyu[yorsa] rahatsız etme.", "If he is sleeping, don't disturb him.")]),
        l("-DIysA / -mIşsA", "Condition on a past action", "B2", "V + DI + ysa / yse + person · V + mIş + sa / se + person",
          "Past + conditional copula: ‘if (it is the case that) … did / has done’. -mIşsA is used when the speaker has no direct knowledge.",
          "This is not the unreal condition, which is -sAydI.",
          [("Ödevini bitir[diyse]n çıkabilirsin.", "If you have finished your homework, you may go out."),
           ("Git[tiyse] artık dönmez.", "If he has gone, he won't come back now."),
           ("Seni kır[mışsa]m özür dilerim.", "If I have hurt you, I apologise.")]),
        l("-(y)AcAksA", "Condition on an intention", "B2", "V + (y)acak / (y)ecek + sa / se + person",
          "Future + conditional copula: ‘if … is going to’.",
          "Often with madem or eğer.",
          [("Gel[ecekse]n haber ver.", "If you are going to come, let me know."),
           ("Yağmur yağ[acaksa] şemsiye alalım.", "If it is going to rain, let's take an umbrella."),
           ("Böyle yap[acaksa]n hiç başlama.", "If you are going to do it like this, don't start at all.")]),
    ]
}

fn voice() -> Vec<NodeDef> {
    vec![
        l("İşteş + Ettirgen + Edilgen", "Stacked voices", "C1", "V + (I)ş + DIr + Il",
          "Voice suffixes stack in a fixed order: reflexive or reciprocal first, then causative, then passive.",
          "gör-üş-tür-ül- ‘be brought together’, tanı-ş-tır-ıl- ‘be introduced’, bar-ış-tır-ıl- ‘be reconciled’.",
          [("İki taraf gör[üştürül]dü.", "The two sides were brought together for talks."),
           ("Yeni müdürle tanı[ştırıl]dık.", "We were introduced to the new director."),
           ("Küsler barı[ştırıl]dı.", "Those who had fallen out were reconciled.")]),
    ]
}

fn negation() -> Vec<NodeDef> {
    vec![
        l("Çift Olumsuzluk", "Double negation", "C1", "V + mA + tense + değil · yok değil",
          "A negative verb followed by değil makes a guarded positive: ‘it is not that … not’.",
          "-mIyor değil, -mAyAcAk değil, -mAmIş değil, -mAz değil; yok değil ‘there is some’; imkânsız değil.",
          [("Seni anlamıyor [değilim].", "It is not that I don't understand you."),
           ("Bunu bilmiyor [değil].", "He is not unaware of this."),
           ("Param yok [değil], ama harcamak istemiyorum.", "It is not that I have no money, but I don't want to spend it.")]),
    ]
}

fn aspect() -> Vec<NodeDef> {
    vec![
        l("-mAklA kal-", "do no more than", "C1", "V + makla / mekle + kal-",
          "Infinitive + instrumental + kalmak: the action stops at this point and goes no further.",
          "Often with yalnızca or sadece. Negative: -mAklA kalmadı ‘did not stop at’.",
          [("Gülümse[mekle kaldı].", "He merely smiled and did nothing more."),
           ("Söz ver[mekle kalmadı], yardım da etti.", "He did not stop at promising; he helped as well."),
           ("Bak[makla kaldık].", "We were left just looking.")]),
    ]
}

fn converbs() -> Vec<NodeDef> {
    vec![
        l("-mAyIp", "instead of / not … but", "B1", "V + mayıp / meyip",
          "Negative -mA + buffer y + -Ip. The first action does not happen; the second happens in its place.",
          "The following verb carries tense and person. -mAyIp dA adds contrast.",
          [("Okula git[meyip] parkta oynamış.", "Instead of going to school, he apparently played in the park."),
           ("Otobüse bin[meyip] yürüdük.", "We didn't take the bus but walked."),
           ("Hiçbir şey söyle[meyip] sustu.", "He said nothing and stayed silent.")]),
    ]
}

fn participles() -> Vec<NodeDef> {
    vec![
        l("-mIş / -(y)AcAk / -mAktA olan", "Compound subject participles", "C1", "V + mIş / (y)AcAk / mAktA + olan + noun",
          "A tense-marked form + olan gives the relative clause an explicit tense or aspect that plain -(y)An lacks.",
          "-mIş olan ‘who has done’, -(y)AcAk olan ‘who is to do’, -mAktA olan ‘who is in the process of doing’.",
          [("Sınavı kazan[mış olan] öğrenciler kayıt yaptırabilir.", "Students who have passed the exam may register."),
           ("Yarın gel[ecek olan] misafirler için oda hazırladık.", "We prepared a room for the guests who are to come tomorrow."),
           ("Art[makta olan] fiyatlar herkesi etkiliyor.", "The rising prices are affecting everyone.")]),
        l("-mIş / -(y)AcAk / -mAktA olduğu", "Compound object participles", "C1", "V + mIş / (y)AcAk / mAktA + ol + DIk + possessive",
          "A tense-marked form + olduğu marks tense or aspect inside an object relative clause or a noun clause.",
          "-mIş olduğu ‘that he has done’, -mAktA olduğu ‘that he is doing’. Formal register.",
          [("Yaz[mış olduğunuz] mektubu aldım.", "I received the letter you have written."),
           ("Sorunun büyü[mekte olduğunu] görüyoruz.", "We see that the problem is growing."),
           ("Gel[ecek olduğunu] bilmiyordum.", "I didn't know that he was to come.")]),
    ]
}

fn adverbial_clauses() -> Vec<NodeDef> {
    vec![
        l("-DIğInA göre", "since / judging by the fact that", "C1", "V + DIk / (y)AcAk + possessive + (n)A + göre",
          "Participle + possessive + dative + göre: draws a conclusion from an observed fact.",
          "-(y)AcAğInA göre for a future fact. Compare madem.",
          [("Işıklar yan[dığına göre] evdeler.", "Since the lights are on, they are at home."),
           ("Sen de gel[eceğine göre] araba alalım.", "Since you are coming too, let's take the car."),
           ("Cevap verme[diğine göre] kızmış olmalı.", "Since he hasn't replied, he must be angry.")]),
        l("-mAsI durumunda / hâlinde", "in the event of", "C1", "V + ma + possessive + durumunda / hâlinde",
          "Short infinitive + possessive + durumunda or hâlinde: a formal conditional, common in rules and contracts.",
          "Synonym: -DIğI takdirde.",
          [("Yangın çık[ması durumunda] asansörü kullanmayınız.", "In the event of fire, do not use the lift."),
           ("Geç kal[manız hâlinde] bize haber verin.", "In case you are late, inform us."),
           ("Ödeme yapıl[maması durumunda] abonelik iptal edilir.", "In the event of non-payment, the subscription is cancelled.")]),
        l("-mAsIndAn önce / sonra", "before / after / from someone's …-ing", "C1", "V + ma + possessive + n + DAn + önce / sonra / itibaren",
          "Short infinitive + possessive + ablative + postposition. Unlike -mAdAn önce it names the subject through the possessive. Also with -(y)Iş: gelişinden sonra.",
          "Formal register.",
          [("Toplantının başla[masından önce] herkes yerini aldı.", "Before the meeting began, everyone took their place."),
           ("Onun git[mesinden sonra] ev çok sessizleşti.", "After his leaving, the house became very quiet."),
           ("İşe başla[masından itibaren] hiç izin almadı.", "From the time he started work, he has taken no leave.")]),
        l("-(y)AcAk kadar", "enough to / too … to", "C1", "V + (y)acak / (y)ecek + kadar",
          "Future participle + kadar: a degree sufficient to bring about a result.",
          "-(y)AmAyAcAk kadar ‘too … to’. Related: -(y)AcAk derecede.",
          [("Bunu anla[yacak kadar] büyüdün.", "You are old enough to understand this."),
           ("Konuşama[yacak kadar] yorgunum.", "I am too tired to speak."),
           ("Herkese yet[ecek kadar] yemek var.", "There is enough food for everyone.")]),
        l("-DIğI süresince", "for as long as / throughout", "C1", "V + DIk + possessive + süresince / müddetçe",
          "Participle + possessive + a duration word: the whole time that something lasts.",
          "Compare -DIğI sürece, which is more conditional.",
          [("Burada kal[dığınız süresince] size yardımcı olacağız.", "We will assist you throughout your stay here."),
           ("Yaşa[dığım müddetçe] bunu unutmayacağım.", "As long as I live I will not forget this."),
           ("Toplantı sür[düğü süresince] telefonlar kapalıydı.", "Phones were off for the duration of the meeting.")]),
    ]
}

fn reported_speech() -> Vec<NodeDef> {
    vec![
        l("diye (doğrudan anlatım)", "Quoting with diye", "A2", "“quotation” + diye + sor- / bağır- / düşün- / cevap ver-",
          "diye (from de- ‘say’) follows the exact words or thought and links them to a verb other than de-. With de- itself no diye is needed.",
          "Also for names: Ali diye biri ‘someone called Ali’. Its purpose and reason uses come at a later level.",
          [("“Geliyorum” [diye] bağırdı.", "He shouted, “I'm coming.”"),
           ("“Neden geç kaldın?” [diye] sordu.", "She asked, “Why are you late?”"),
           ("“Hayır” [diye] cevap verdi.", "He answered, “No.”")]),
        l("Emir Kipinde Dolaylı Anlatım", "Reported commands and requests", "B1", "V + ma + possessive + (n)I + söyle- / iste- / rica et-",
          "An order or request is reported with the short infinitive + possessive + accusative. The possessive shows who is to act.",
          "Direct: “Gel!” dedi → indirect: Gelmemi söyledi. Negative: gelmememi söyledi.",
          [("Doktor sigarayı bırak[mamı] söyledi.", "The doctor told me to give up smoking."),
           ("Annem erken gel[memizi] istedi.", "My mother asked us to come early."),
           ("Öğretmen sessiz ol[malarını] rica etti.", "The teacher asked them to be quiet.")]),
        l("İstek Kipinde Dolaylı Anlatım", "Reported suggestions and offers", "B2", "V + mAk istediğini söyle- · V + mAyI teklif et- / öner-",
          "“-(y)AlIm” and “-(y)AyIm” are reported with -mAk istediğini söyle-, or with -mAyI teklif et- / öner-.",
          "Direct: “Sinemaya gidelim” dedi → indirect: Sinemaya gitmeyi teklif etti.",
          [("Sinemaya git[meyi teklif etti].", "He suggested going to the cinema."),
           ("Bana yardım et[mek istediğini] söyledi.", "She said that she wanted to help me."),
           ("Erken yola çık[mamızı önerdi].", "He suggested that we set off early.")]),
        l("İsim Cümlelerinde Dolaylı Anlatım", "Reporting ‘to be’ sentences", "B2", "N / Adj + ol + DIk / (y)AcAk + possessive + (n)I + söyle-",
          "A nominal sentence is reported with olduğunu (present or past) or olacağını (future). var and yok become olduğunu and olmadığını.",
          "Direct: “Hastayım” dedi → Hasta olduğunu söyledi. “Param yok” → Parası olmadığını söyledi.",
          [("Hasta [olduğunu] söyledi.", "He said that he was ill."),
           ("Yarın evde [olacağını] söyledi.", "She said that she would be at home tomorrow."),
           ("Parası [olmadığını] söyledi.", "He said that he had no money.")]),
    ]
}

fn questions() -> Vec<NodeDef> {
    vec![
        l("İmalı Soru", "Insinuating and rhetorical questions", "C1", "statement + mI (not a real question)",
          "A question form that asserts the opposite, or expresses doubt, reproach or surprise. Stress and context carry the meaning.",
          "Ben mi söyledim? ‘Was it I who said it?’ (= I didn't). Olur mu hiç? ‘How could that be?’ Bilmez miyim! ‘Don't I know it!’",
          [("Bunu ben [mi] yaptım?", "Was it me who did this? (I didn't.)"),
           ("Hiç böyle şey olur [mu]?", "How could such a thing be?"),
           ("Seni unutur [muyum]?", "Would I ever forget you?")]),
    ]
}

fn meaning() -> NodeDef {
    b("Anlam Bilgisi", "Meaning & Usage", "B2–C1",
      "How words carry meaning beyond the literal, and common faults of expression.",
      vec![
        l("Mecaz Anlam", "Figurative meaning", "C1", "gerçek anlam → mecaz anlam",
          "A word used outside its literal (gerçek) meaning. Body-part and temperature words are especially productive.",
          "soğuk ‘cold’ → ‘unfriendly’; ağır ‘heavy’ → ‘serious, slow’; tatlı ‘sweet’ → ‘pleasant’; kırmak ‘break’ → ‘hurt someone's feelings’; baş ‘head’ → ‘leader, beginning’.",
          [("Bana çok [soğuk] davrandı.", "He behaved very coldly towards me."),
           ("Kalbimi [kırdın].", "You broke my heart."),
           ("[Tatlı] bir sohbet ettik.", "We had a pleasant chat.")]),
        l("Ad Aktarması", "Metonymy", "C1", "container → content · place → people · author → work",
          "A thing is named by something closely connected with it.",
          "sobayı yak- (the fuel in it), bütün okul (the people), Ankara (the government), Yunus Emre oku- (his poems), bir tabak ye- (the food).",
          [("[Ankara] bu konuda henüz açıklama yapmadı.", "Ankara has not yet made a statement on this."),
           ("Bütün [sınıf] güldü.", "The whole class laughed."),
           ("İki [tabak] yedi.", "He ate two platefuls.")]),
        l("Deyimler", "Idioms", "B2", "fixed phrase, usually N + V",
          "Fixed expressions with a non-literal meaning. The words cannot be replaced, but the verb conjugates normally.",
          "göz atmak ‘glance’, kulak vermek ‘listen’, el atmak ‘take in hand’, ayak uydurmak ‘keep up’, burnunun dikine gitmek ‘be headstrong’, etekleri zil çalmak ‘be overjoyed’.",
          [("Gazeteye bir [göz attım].", "I glanced at the newspaper."),
           ("Yeni işine çabuk [ayak uydurdu].", "He quickly adapted to his new job."),
           ("Sözlerime [kulak ver].", "Heed my words.")]),
        l("Atasözleri", "Proverbs", "C1", "complete sentence, usually aorist or imperative",
          "Fixed sentences of traditional advice. Unlike idioms they are complete judgements and do not conjugate.",
          "Bir elin nesi var, iki elin sesi var. Dost kara günde belli olur. Akıl yaşta değil baştadır.",
          [("[Damlaya damlaya göl olur.]", "Drop by drop a lake forms: little by little adds up."),
           ("[Ağaç yaşken eğilir.]", "A tree is bent while it is young: teach early."),
           ("[Sakla samanı, gelir zamanı.]", "Keep the straw; its time will come: waste not, want not.")]),
        l("Anlatım Bozukluğu", "Faults of expression", "C1", "gereksiz sözcük · çelişki · eksik öge · özne–yüklem uyumsuzluğu",
          "Typical errors: a redundant word, contradictory words, an object or complement wrongly shared between two verbs, and subject–verb disagreement.",
          "Redundant: geri iade etmek (iade already means ‘give back’). Contradiction: kesinlikle … olabilir. Missing complement: onu sever ve güvenirim (güven- needs ona).",
          [("✗ Kitabı [geri iade] ettim. → Kitabı iade ettim.", "I returned the book: geri is redundant."),
           ("✗ [Kesinlikle] gelmiş [olabilir]. → Gelmiş olabilir.", "He may have come: certainty and possibility clash."),
           ("✗ Onu sever ve [güvenirim]. → Onu sever, ona güvenirim.", "I love him and trust him: güven- needs the dative.")]),
      ])
}
