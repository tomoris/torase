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
  - entity.n.01 -> abstraction.n.06 -> relation.n.01 -> * (part.n.01 -> substance.n.01 以外)
- `ro[CV]+`
  - entity.n.01 -> abstraction.n.06 -> communication.n.02
- `ru[CV]+`
  - entity.n.01 -> abstraction.n.06 -> * (psychological_feature.n.01, group.n.01, attribute.n.02, relation.n.01, communication.n.02 以外)

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
  - x1: `bV`, x2: `gV|mV|rV`
- `ki[CV]+` (生物以外の物理的実体x1が抽象概念x2をxxxする)
  - x1: `dV|nV`, x2: `gV|mV|rV`
- `ku[CV]+` (抽象概念x1が抽象概念x2をxxxする)
  - x1: `gV|mV|rV`, x2: `gV|mV|rV`

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

## 機能語

### 疑問文マーカー

- `ku`
- `?` (読み、発音は`ku`)

### 様相論理

#### 量化力

- □ 必然
- ◇ 可能

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

??

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

### その他

- `si` (第一項と第二項の区切り)
- `re` (述語の第一項と第二項の入れ替え)
- `ro` (項抽出)
- `se` (命題を項に変換、命題の始まりに置く)
- `so` (命題を項に変換、命題の終わりに置く)

# 設計意図

エスペラントのように1つの語根から名詞や動詞などを派生させると、語の表層文字列が変化してしまうのでこの方向性は採用せず、ロジバンを参考に述語を用いて表現することにした。
計画言語を学習上で最も労力がかかるのが内容語を覚えることである。
ロジバンではn項述語を覚える労力がかかるが、この計画言語では単項述語、二項述語に制限し、項の取りうる型で分類した。
名詞を表す述語では第一音節ごとに分類規則を設けることで覚えやすく、また初見の語でも意味が類推しやすいように設計した。
語の恣意性の問題から、内容語の指す概念や発音を既存言語を参考にするか、独立に設計するかの選択肢があったが、現状最も国際語として使用されている状況や、言語学の知見やリソースの豊富さから英語を参考に内容語を設計した。
