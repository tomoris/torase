//! 項の選択制限に使う意味論的な型。表層形(語頭音節)から一意に決まる
//! (`tokenizer::BinaryPredicate::get_argument_types`/`Argument::get_type`参照)ため、
//! `argument_type_checker`と`tokenizer`の双方から参照される。

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ArgumentType {
    PhysicalEntity(PhysicalEntity),
    Abstraction,
    Space, // SpatialRelation, Distanceから構成
    Time,  // TemporalRelation, Durationから構成
    Epsilon,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PhysicalEntity {
    BLivingThing,
    DArtifact,
    NOther,
}

impl std::fmt::Display for ArgumentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing) => "生物",
            ArgumentType::PhysicalEntity(PhysicalEntity::DArtifact) => "人工物",
            ArgumentType::PhysicalEntity(PhysicalEntity::NOther) => "その他の物理的実体",
            ArgumentType::Abstraction => "抽象",
            ArgumentType::Space => "空間",
            ArgumentType::Time => "時間",
            ArgumentType::Epsilon => "項なし",
        };
        f.write_str(name)
    }
}
