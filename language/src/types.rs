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
