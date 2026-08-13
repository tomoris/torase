# 品詞

- 内容語
  - 述語
    - 単項述語
    - 二項述語
  - 項
- 機能語

## 内容語

名詞、形容詞・副詞などの修飾詞は述語から派生させる。

### 述語

二項述語と単項述語に分かれているが、単項述語は統語論において二番目の項にεが入ってる二項述語とみなす。
述語はhとs以外の子音から始まり、二音節以上持つ。

#### 名詞を表す述語

f(x1, x2)という二項述語で表現され、x1はx2をxxと定義するという命題を表す。
WordNetの階層構造に従い、第一音節が決まり、第一音節の子音は有声音である。

- `ba[CV]+` (人に関する名詞)
  - entity.n.01 -> physical_entity.n.01 -> object.n.01 -> whole.n.02 -> living_thing.n.01 -> organism.n.01 -> person.n.01
- `be[CV]+` (植物に関する名詞)
  - entity.n.01 -> physical_entity.n.01 -> object.n.01 -> whole.n.02 -> living_thing.n.01 -> organism.n.01 -> plant.n.02
- `bi[CV]+` (動物に関する名詞)
  - entity.n.01 -> physical_entity.n.01 -> object.n.01 -> whole.n.02 -> living_thing.n.01 -> organism.n.01 -> animal.n.01
- `bu[CV]+` (その他生物に関する名詞)
  - entity.n.01 -> physical_entity.n.01 -> object.n.01 -> whole.n.02 -> living_thing.n.01 ->  * (organism.n.01 -> person.n.01, organism.n.01 -> plant.n.02, organism.n.01 -> animal.n.01 以外)
- `da[CV]+`
  - entity.n.01 -> physical_entity.n.01 -> object.n.01 -> whole.n.02 -> artifact.n.01 -> instrumentality.n.03
- `di[CV]+`
  - entity.n.01 -> physical_entity.n.01 -> object.n.01 -> whole.n.02 -> artifact.n.01 -> structure.n.01
- `du[CV]+`
  - entity.n.01 -> physical_entity.n.01 -> object.n.01 -> whole.n.02 -> artifact.n.01 -> * (instrumentality.n.03, structure.n.01 以外)
- `na[CV]+`
  - entity.n.01 -> physical_entity.n.01 -> object.n.01 -> whole.n.02 -> * (living_thing.n.01, artifact.n.01 以外)
- `ne[CV]+`
  - entity.n.01 -> physical_entity.n.01 -> object.n.01 -> * (whole.n.02 以外)
- `ni[CV]+`
  - entity.n.01 -> physical_entity.n.01 -> matter.n.03 -> substance.n.01
- `no[CV]+`
  - entity.n.01 -> physical_entity.n.01 -> matter.n.03 -> * (substance.n.01 以外)
- `nu[CV]+`
  - entity.n.01 -> physical_entity.n.01 -> * (object.n.01, causal_agent.n.01, matter.n.03 以外)
- `ga[CV]+`
  - entity.n.01 -> abstraction.n.06 -> psychological_feature.n.01 -> event.n.01 -> act.n.02 -> activity.n.01
- `ge[CV]+`
  - entity.n.01 -> abstraction.n.06 -> psychological_feature.n.01 -> event.n.01 -> act.n.02 -> * (activity.n.01 以外)
- `gi[CV]+`
  - entity.n.01 -> abstraction.n.06 -> psychological_feature.n.01 -> event.n.01 -> * (act.n.02 以外)
- `gu[CV]+`
  - entity.n.01 -> abstraction.n.06 -> psychological_feature.n.01 -> * (event.n.01 以外)
- `ma[CV]+`
  - entity.n.01 -> abstraction.n.06 -> group.n.01 -> biological_group.n.01 -> taxonomic_group.n.01 -> genus.n.02
- `mi[CV]+`
  - entity.n.01 -> abstraction.n.06 -> group.n.01 -> biological_group.n.01 -> * (taxonomic_group.n.01 -> genus.n.02 以外)
- `mu[CV]+`
  - entity.n.01 -> abstraction.n.06 -> group.n.01 -> * (biological_group.n.01 以外)
- `ra[CV]+`
  - entity.n.01 -> abstraction.n.06 -> attribute.n.02 -> state.n.02
- `re[CV]+`
  - entity.n.01 -> abstraction.n.06 -> attribute.n.02 -> * (state.n.02 以外)
- `ri[CV]+`
  - entity.n.01 -> abstraction.n.06 -> relation.n.01 -> * (part.n.01 -> substance.n.01, spatial_relation.n.01, temporal_relation.n.01 以外)
- `ro[CV]+`
  - entity.n.01 -> abstraction.n.06 -> communication.n.02
- `ru[CV]+`
  - entity.n.01 -> abstraction.n.06 -> * (psychological_feature.n.01, group.n.01, attribute.n.02, relation.n.01, communication.n.02, measure.n.02 -> time_period.n.01, measure.n.02 -> linear_measure.n.01 以外)

##### 空間・時間を表す名詞述語(z系)

空間・時間に関わる概念はWordNetでは`abstraction.n.06`(関係・尺度)に存在するが、`heni`/`hene`/`heno`(空間格)・`hegi`/`hege`/`hego`(時間格)の型検査で他の関係・尺度概念(親族関係、因果関係、重さ・体積の尺度など)と混同しないよう、専用の音節グループとして独立させている。

- `za[CV]+` (空間関係。例:「中」「上」「近く」)
  - abstraction.n.06 -> relation.n.01 -> spatial_relation.n.01
- `zi[CV]+` (時間関係。例:「前」「後」「間」)
  - abstraction.n.06 -> relation.n.01 -> temporal_relation.n.01
- `zu[CV]+` (時間の尺度。例:「日」「分」「年」)
  - abstraction.n.06 -> measure.n.02 -> time_period.n.01
- `ze[CV]+` (空間の尺度。例:「メートル」)
  - abstraction.n.06 -> measure.n.02 -> linear_measure.n.01

`za`/`zi`/`zu`/`ze`はそれぞれ`SpatialRelation`/`TemporalRelation`/`Duration`/`Distance`型として扱われる。

#### 動詞を表す述語

f(x1)もしくはf(x1, x2)という形式で表され、x1は???をする(自動詞)もしくはx1はx2を対象にとりxxをする(他動詞)という命題を表す。
x1, x2に取れる項は、名詞を表す述語で用いた第一音節の分類に基づく。

- `pa[CV]+` (生物x1が生物x2をxxxする)
  - x1: `bV`, x2: `bV`
- `pe[CV]+` (生物x1が生物以外の物理的実体x2をxxxする)
  - x1: `bV`, x2: `dV|nV`
- `pi[CV]+`, (生物x1がxxxする)
  - x1: `bV`
- `po[CV]+` (生物以外の物理的実体x1が生物以外の物理的実体x2をxxxする)
  - x1: `dV|nV`, x2: `dV|nV`
- `pu[CV]+`, (生物以外の物理的実体x1がxxxする)
  - x1: `dV|nV`
- `ka[CV]+` (生物x1が抽象概念x2をxxxする)
  - x1: `bV`, x2: `gV|mV|rV|zV`
- `ki[CV]+` (生物以外の物理的実体x1が抽象概念x2をxxxする)
  - x1: `dV|nV`, x2: `gV|mV|rV|zV`
- `ku[CV]+` (抽象概念x1が抽象概念x2をxxxする)
  - x1: `gV|mV|rV|zV`, x2: `gV|mV|rV|zV`

### 項

人称代名詞

- `mi` (私)
- `ma` (私とあなたたち)
- `mu` (あなたを含まない私たち)
- `ba` (あなた)
- `bu` (あなたたち)
- `ga` (誰か、単数か複数か指定しない)
- `gi` (彼/彼女)
- `gu` (彼ら)

指示代名詞

- `me` (これ)
- `be` (それ)
- `ge` (あれ)

指示代名詞は指示物の種類を限定しないため、生物・人工物・その他物理的実体・抽象概念に加えて、空間(`heni`/`hene`/`heno`格の「ここ/そこ/あそこ」)や時間(`hegi`/`hege`/`hego`格の「いま/そのとき/あのとき」)も指すことができる。

## 機能語

### 疑問文マーカー

- `ku`
- `?` (読み、発音は`ku`)

### 様相論理

#### 量化力

- `za` (必然)
 - □ と書いてもいい(読み、発音は`za`)
- `zi` (可能)
  - ◇ と書いてもいい(読み、発音は`zi`)

##### モーダルベース

- `hapa` (認識)
  - 量化対象：話し手が知っている事実と矛盾しない世界 w_{epi}
  - hapa □ e (確実にe)
  - hapa ◇ e (eかもしれない)
- `hapi` (義務)
  - 量化対象：法律・規則・道徳を守っている世界 w_{deon}
  - hapi □ e (eしなければならない)
  - hapi ◇ e (eしてもよい)
- `hapu` 能力・状況
  - 量化対象：現実の能力・環境条件を保持した世界 w_{cir}
  - hapu □ e (eを必ずできる)
  - hapu ◇ e (eすることができる)

#### テンス

- `hita` (過去)
- `hiti` (現在)
- `hitu` (未来)

#### アスペクト

- `huda` (未然)
- `hude` (開始)
- `hudida` (進行)
- `hudidi` (習慣)
- `hudidu` (反復)
- `hudo` (終了)
- `hudu` (事後)

### 否定

- `hane` (モダリティの否定)
- `hine` (テンスの否定)
- `hune` (アスペクトの否定)
- `ne` (述語、項の否定)

### 演算子

- 命題
  - `homa` (and)
  - `homi` (or)
  - `homu` (xor)
- 述語
  - `hoba` (and)
  - `hobi` (or)
  - `hobu` (xor)
- 項
  - `hoka` (and)
  - `hoki` (or)
  - `hoku` (xor)

### 量化詞

- `ta` (すべての)
- `te` (多くの)
- `ti` (半分の)
- `to` (少しの)

### 限定詞

- `mo` (この)
- `bo` (その)
- `go` (あの)

### 修飾句の区切り

- `da` (命題修飾句の前置詞)
- `di` (述語修飾句の前置詞)
- `du` (項修飾句の前置詞)

### 格

述語の項の直前に置き、意味役割を示す。`sa`(主格)・`si`(対格)は二項述語の項構造(x1, x2)に対応する必須格で、それ以外は付加詞(省略可・複数回の付与は不可)。

- `sa` (主格) — 述語のx1(第一項)
- `si` (対格) — 述語のx2(第二項)

付加詞の格は意味領域ごとにグループ化される。

生物系(だれ)

- `subu` (随伴格) — だれと
- `sube` (奪格・人間起点格) — だれから、だれの
- `subo` (与格・人間目標格) — だれまで、だれに

空間系(どこ)

- `suni` (空間格) — どこで
- `sune` (空間起点格) — どこから
- `suno` (空間目標格) — どこまで

時間系(いつ)

- `sugi` (時間格) — いつ
- `suge` (時間起点格) — いつから
- `sugo` (時間目標格) — いつまで

道具系

- `sudu` (具格) — 何々を使って

抽象系

- `sute` (原因格・理由起点格) — 何々のせいで
- `suto` (目的格・理由目標格) — 何々のために

#### 格ごとの項の型

- `sa`/`si`: 述語の`get_argument_types`が返す(x1, x2)の型
- `subu`/`sube`/`subo`: `bV`(生物)
- `sudu`: `dV`(人工物。`da`/`di`/`du`は現状区別していない)
- `sute`/`suto`: `gV|mV|rV|zV`(抽象概念全般。`ka`/`ki`/`ku`の抽象項と同じ範囲)
- `suni`/`sune`/`suno`: 空間を指す指示代名詞(`me`/`be`/`ge`)、または`za`(空間関係。例:「Xの中」「Xの近く」)
- `sugi`/`suge`/`sugo`: 時を指す指示代名詞(`me`/`be`/`ge`)、`ziV`(時間関係)、または`zu`(時間の尺度。例:「6日」「5分間」)

### 接続詞

- `hesute` (結果 -> 原因) - P1した なぜなら P2だから
- `henesute` (結果 -> 反対の原因) - P1したP2なのにもかかわらず
- `hesuto` (原因 -> 結果) - P1なのでP2する
- `henesuto` (反対の原因 -> 結果) - P1なのにもかかわらずP2する
- `hehoma` (and) - P1並びにP2する
- `hehomi` (or) - P1もしくはP2する
- `hehomu` (xor) - P1もしくはP2のどちらか片方だけする
- `hehita` (時間前) - P1の前にP2する
- `hehiti` (同時) - P1、P2を同時にする
- `hehitu` (時間後) - P1の後にP2する
- `hezu` (条件) - P1ならP2する
- `hehema` (例示) - P1例えばP2
- `hehegi` (言い換え) - P1つまりP2
- `hehenu` (対比) - P1、一方P2

### その他

- `ra`/`ri` (項抽出。それぞれ述語の第一項・第二項を抽出する)
- `he` (命題を項に変換、命題の始まりに置く)
- `ho` (命題を項に変換、命題の終わりに置く)
- `se` (型オーバーライド。格標識(`sa`/`si`/`su[CV]`)の直後・項句の直前に置く任意のトークンで、その項だけを述語の型検査から除外し、任意の型として扱わせる。比喩・見立て表現用。例:「parobe(x1はx2を愛す。x1,x2ともに生物`bV`を要求)sa se ri ditabare si ba」で、本来生物ではない項を主格に置ける)

# 設計意図

エスペラントのように1つの語根から名詞や動詞などを派生させると、語の表層文字列が変化してしまうのでこの方向性は採用せず、ロジバンを参考に述語を用いて表現することにした。
計画言語を学習上で最も労力がかかるのが内容語を覚えることである。
ロジバンではn項述語を覚える労力がかかるが、この計画言語では単項述語、二項述語に制限し、項の取りうる型で分類した。
名詞を表す述語では第一音節ごとに分類規則を設けることで覚えやすく、また初見の語でも意味が類推しやすいように設計した。
語の恣意性の問題から、内容語の指す概念や発音を既存言語を参考にするか、独立に設計するかの選択肢があったが、現状最も国際語として使用されている状況や、言語学の知見やリソースの豊富さから英語を参考に内容語を設計した。

なお`se`は上記の音韻による型推定の仕組みを弱めるものではなく、比喩・見立て表現のために型検査を明示的にバイパスする例外機構として導入したものであり、通常の文では使用しないことを想定している。
