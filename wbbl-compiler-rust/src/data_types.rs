use crate::constraint_solver_constraints::CompositeSize::*;
use crate::constraint_solver_constraints::Dimensionality::*;
use crate::constraint_solver_constraints::*;
use serde::{Deserialize, Serialize};
use AbstractDataType::*;
use ConcreteDataType::*;

#[derive(Serialize, Deserialize, Copy, Clone, PartialOrd, PartialEq, Hash, Ord, Eq)]
pub enum ComputationDomain {
    TimeVarying,
    ModelDependant,
    TransformDependant,
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialOrd, PartialEq, Hash, Ord, Eq)]
pub enum AbstractDataType {
    Any,
    AnyFloat,
    AnyField,
    AnyMaterial,

    AnyFieldWithDimensionality(Dimensionality),
    AnyFieldWithCompositeSize(CompositeSize),

    AnyTexture,
    AnyTextureWithDimensionality(Dimensionality),
    AnyTextureWithCompositeSize(CompositeSize),

    AnyProceduralField,
    AnyProceduralFieldWithDimensionality(Dimensionality),
    AnyProceduralFieldWithCompositeSize(CompositeSize),

    AnyFloat123,

    ConcreteType(ConcreteDataType),
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialOrd, PartialEq, Hash, Ord, Eq)]
pub enum ConcreteDataType {
    Float(CompositeSize),
    Int,
    Bool,
    Texture(Dimensionality, CompositeSize),
    ProceduralField(Dimensionality, CompositeSize),
    SlabMaterial,
}

impl HasCompositeSize for ConcreteDataType {
    fn get_composite_size(&self) -> Option<CompositeSize> {
        match self {
            Float(c) => Some(*c),
            Int => Some(S1),
            Bool => Some(S1),
            Texture(_, c) => Some(*c),
            ProceduralField(_, c) => Some(*c),
            SlabMaterial => None,
        }
    }
}

impl HasDimensionality for ConcreteDataType {
    fn get_dimensionality(&self) -> Option<Dimensionality> {
        match self {
            Float(S1) => Some(D1),
            Float(S2) => Some(D2),
            Float(S3) => Some(D3),
            Float(S4) => Some(D4),
            Int => Some(D1),
            Bool => Some(D1),
            Texture(d, _) => Some(*d),
            ProceduralField(d, _) => Some(*d),
            SlabMaterial => None,
        }
    }
}

impl HasRanking for ConcreteDataType {
    fn get_rank(&self) -> i32 {
        0
    }
}

impl HasCompositeSize for AbstractDataType {
    fn get_composite_size(&self) -> Option<CompositeSize> {
        match self {
            Any => None,
            AnyFloat => None,
            AnyField => None,
            AnyMaterial => None,
            AnyFieldWithDimensionality(_) => None,
            AnyFieldWithCompositeSize(c) => Some(*c),
            AnyTexture => None,
            AnyTextureWithDimensionality(_) => None,
            AnyTextureWithCompositeSize(c) => Some(*c),
            AnyProceduralField => None,
            AnyProceduralFieldWithDimensionality(_) => None,
            AnyProceduralFieldWithCompositeSize(c) => Some(*c),
            AnyFloat123 => None,
            ConcreteType(t) => t.get_composite_size(),
        }
    }
}

impl HasDimensionality for AbstractDataType {
    fn get_dimensionality(&self) -> Option<Dimensionality> {
        match self {
            Any => None,
            AnyFloat => None,
            AnyField => None,
            AnyMaterial => None,
            AnyFieldWithDimensionality(d) => Some(*d),
            AnyFieldWithCompositeSize(_) => None,
            AnyTexture => None,
            AnyTextureWithDimensionality(d) => Some(*d),
            AnyTextureWithCompositeSize(_) => None,
            AnyProceduralField => None,
            AnyProceduralFieldWithDimensionality(d) => Some(*d),
            AnyProceduralFieldWithCompositeSize(_) => None,
            AnyFloat123 => None,
            ConcreteType(t) => t.get_dimensionality(),
        }
    }
}

impl HasRanking for AbstractDataType {
    fn get_rank(&self) -> i32 {
        match self {
            Any => 0,
            AnyFloat => 1,
            AnyField => 1,
            AnyMaterial => 1,
            AnyFieldWithDimensionality(_) => 2,
            AnyFieldWithCompositeSize(_) => 2,
            AnyTexture => 2,
            AnyTextureWithDimensionality(_) => 3,
            AnyTextureWithCompositeSize(_) => 3,
            AnyProceduralField => 2,
            AnyProceduralFieldWithDimensionality(_) => 3,
            AnyProceduralFieldWithCompositeSize(_) => 3,
            AnyFloat123 => 4,
            ConcreteType(_) => 5,
        }
    }
}

impl AbstractDataType {
    pub fn get_concrete_domain(&self) -> Vec<ConcreteDataType> {
        match self {
            Any => vec![
                Float(S1),
                Float(S2),
                Float(S3),
                Float(S4),
                Int,
                Bool,
                Texture(D1, S1),
                Texture(D1, S2),
                Texture(D1, S3),
                Texture(D1, S4),
                Texture(D2, S1),
                Texture(D2, S2),
                Texture(D2, S3),
                Texture(D2, S4),
                Texture(D3, S1),
                Texture(D3, S2),
                Texture(D3, S3),
                Texture(D3, S4),
                Texture(D4, S1),
                Texture(D4, S2),
                Texture(D4, S3),
                Texture(D4, S4),
                ProceduralField(D1, S1),
                ProceduralField(D1, S2),
                ProceduralField(D1, S3),
                ProceduralField(D1, S4),
                ProceduralField(D2, S1),
                ProceduralField(D2, S2),
                ProceduralField(D2, S3),
                ProceduralField(D2, S4),
                ProceduralField(D3, S1),
                ProceduralField(D3, S2),
                ProceduralField(D3, S3),
                ProceduralField(D3, S4),
                ProceduralField(D4, S1),
                ProceduralField(D4, S2),
                ProceduralField(D4, S3),
                ProceduralField(D4, S4),
                SlabMaterial,
            ],
            AnyFloat => vec![Float(S1), Float(S2), Float(S3), Float(S4)],
            AnyField => vec![
                Texture(D1, S1),
                Texture(D1, S2),
                Texture(D1, S3),
                Texture(D1, S4),
                Texture(D2, S1),
                Texture(D2, S2),
                Texture(D2, S3),
                Texture(D2, S4),
                Texture(D3, S1),
                Texture(D3, S2),
                Texture(D3, S3),
                Texture(D3, S4),
                Texture(D4, S1),
                Texture(D4, S2),
                Texture(D4, S3),
                Texture(D4, S4),
                ProceduralField(D1, S1),
                ProceduralField(D1, S2),
                ProceduralField(D1, S3),
                ProceduralField(D1, S4),
                ProceduralField(D2, S1),
                ProceduralField(D2, S2),
                ProceduralField(D2, S3),
                ProceduralField(D2, S4),
                ProceduralField(D3, S1),
                ProceduralField(D3, S2),
                ProceduralField(D3, S3),
                ProceduralField(D3, S4),
                ProceduralField(D4, S1),
                ProceduralField(D4, S2),
                ProceduralField(D4, S3),
                ProceduralField(D4, S4),
            ],
            AnyMaterial => vec![SlabMaterial],
            AnyFieldWithDimensionality(d) => vec![
                Texture(*d, S1),
                Texture(*d, S2),
                Texture(*d, S3),
                Texture(*d, S4),
                ProceduralField(*d, S1),
                ProceduralField(*d, S2),
                ProceduralField(*d, S3),
                ProceduralField(*d, S4),
            ],
            AnyFieldWithCompositeSize(c) => vec![
                Texture(D1, *c),
                Texture(D2, *c),
                Texture(D3, *c),
                Texture(D4, *c),
                ProceduralField(D1, *c),
                ProceduralField(D2, *c),
                ProceduralField(D3, *c),
                ProceduralField(D4, *c),
            ],
            AnyTexture => vec![
                Texture(D1, S1),
                Texture(D1, S2),
                Texture(D1, S3),
                Texture(D1, S4),
                Texture(D2, S1),
                Texture(D2, S2),
                Texture(D2, S3),
                Texture(D2, S4),
                Texture(D3, S1),
                Texture(D3, S2),
                Texture(D3, S3),
                Texture(D3, S4),
                Texture(D4, S1),
                Texture(D4, S2),
                Texture(D4, S3),
                Texture(D4, S4),
            ],
            AnyTextureWithDimensionality(d) => vec![
                Texture(*d, S1),
                Texture(*d, S2),
                Texture(*d, S3),
                Texture(*d, S4),
            ],
            AnyTextureWithCompositeSize(c) => vec![
                Texture(D1, *c),
                Texture(D2, *c),
                Texture(D3, *c),
                Texture(D4, *c),
            ],
            AnyProceduralField => vec![
                ProceduralField(D1, S1),
                ProceduralField(D1, S2),
                ProceduralField(D1, S3),
                ProceduralField(D1, S4),
                ProceduralField(D2, S1),
                ProceduralField(D2, S2),
                ProceduralField(D2, S3),
                ProceduralField(D2, S4),
                ProceduralField(D3, S1),
                ProceduralField(D3, S2),
                ProceduralField(D3, S3),
                ProceduralField(D3, S4),
                ProceduralField(D4, S1),
                ProceduralField(D4, S2),
                ProceduralField(D4, S3),
                ProceduralField(D4, S4),
            ],
            AnyProceduralFieldWithDimensionality(d) => vec![
                ProceduralField(*d, S1),
                ProceduralField(*d, S2),
                ProceduralField(*d, S3),
                ProceduralField(*d, S4),
            ],
            AnyProceduralFieldWithCompositeSize(c) => vec![
                ProceduralField(D1, *c),
                ProceduralField(D2, *c),
                ProceduralField(D3, *c),
                ProceduralField(D4, *c),
            ],
            AnyFloat123 => vec![Float(S1), Float(S2), Float(S3)],
            ConcreteType(t) => vec![*t],
        }
    }

    pub fn get_abstract_domain(&self) -> Vec<AbstractDataType> {
        let mut domain = match self {
            Any => vec![
                Any,
                AnyFloat,
                AnyField,
                AnyMaterial,
                AnyFieldWithDimensionality(D1),
                AnyFieldWithDimensionality(D2),
                AnyFieldWithDimensionality(D3),
                AnyFieldWithDimensionality(D4),
                AnyFieldWithCompositeSize(S1),
                AnyFieldWithCompositeSize(S2),
                AnyFieldWithCompositeSize(S3),
                AnyFieldWithCompositeSize(S4),
                AnyTexture,
                AnyTextureWithDimensionality(D1),
                AnyTextureWithDimensionality(D2),
                AnyTextureWithDimensionality(D3),
                AnyTextureWithDimensionality(D4),
                AnyTextureWithCompositeSize(S1),
                AnyTextureWithCompositeSize(S2),
                AnyTextureWithCompositeSize(S3),
                AnyTextureWithCompositeSize(S4),
                AnyProceduralField,
                AnyProceduralFieldWithDimensionality(D1),
                AnyProceduralFieldWithDimensionality(D2),
                AnyProceduralFieldWithDimensionality(D3),
                AnyProceduralFieldWithDimensionality(D4),
                AnyProceduralFieldWithCompositeSize(S1),
                AnyProceduralFieldWithCompositeSize(S2),
                AnyProceduralFieldWithCompositeSize(S3),
                AnyProceduralFieldWithCompositeSize(S4),
                AnyFloat123,
            ],
            AnyFloat => vec![AnyFloat, AnyFloat123],
            AnyField => vec![
                AnyField,
                AnyFieldWithDimensionality(D1),
                AnyFieldWithDimensionality(D2),
                AnyFieldWithDimensionality(D3),
                AnyFieldWithDimensionality(D4),
                AnyFieldWithCompositeSize(S1),
                AnyFieldWithCompositeSize(S2),
                AnyFieldWithCompositeSize(S3),
                AnyFieldWithCompositeSize(S4),
                AnyTexture,
                AnyTextureWithDimensionality(D1),
                AnyTextureWithDimensionality(D2),
                AnyTextureWithDimensionality(D3),
                AnyTextureWithDimensionality(D4),
                AnyTextureWithCompositeSize(S1),
                AnyTextureWithCompositeSize(S2),
                AnyTextureWithCompositeSize(S3),
                AnyTextureWithCompositeSize(S4),
                AnyProceduralField,
                AnyProceduralFieldWithDimensionality(D1),
                AnyProceduralFieldWithDimensionality(D2),
                AnyProceduralFieldWithDimensionality(D3),
                AnyProceduralFieldWithDimensionality(D4),
                AnyProceduralFieldWithCompositeSize(S1),
                AnyProceduralFieldWithCompositeSize(S2),
                AnyProceduralFieldWithCompositeSize(S3),
                AnyProceduralFieldWithCompositeSize(S4),
            ],
            AnyMaterial => vec![AnyMaterial],
            AnyFieldWithDimensionality(d) => vec![
                AnyFieldWithDimensionality(*d),
                AnyTextureWithDimensionality(*d),
                AnyProceduralFieldWithDimensionality(*d),
            ],
            AnyFieldWithCompositeSize(c) => vec![
                AnyFieldWithCompositeSize(*c),
                AnyTextureWithCompositeSize(*c),
                AnyProceduralFieldWithCompositeSize(*c),
            ],
            AnyTexture => vec![
                AnyTexture,
                AnyTextureWithDimensionality(D1),
                AnyTextureWithDimensionality(D2),
                AnyTextureWithDimensionality(D3),
                AnyTextureWithDimensionality(D4),
                AnyTextureWithCompositeSize(S1),
                AnyTextureWithCompositeSize(S2),
                AnyTextureWithCompositeSize(S3),
                AnyTextureWithCompositeSize(S4),
            ],
            AnyTextureWithDimensionality(d) => vec![AnyTextureWithDimensionality(*d)],
            AnyTextureWithCompositeSize(c) => vec![AnyTextureWithCompositeSize(*c)],
            AnyProceduralField => vec![
                AnyProceduralField,
                AnyProceduralFieldWithDimensionality(D1),
                AnyProceduralFieldWithDimensionality(D2),
                AnyProceduralFieldWithDimensionality(D3),
                AnyProceduralFieldWithDimensionality(D4),
                AnyProceduralFieldWithCompositeSize(S1),
                AnyProceduralFieldWithCompositeSize(S2),
                AnyProceduralFieldWithCompositeSize(S3),
                AnyProceduralFieldWithCompositeSize(S4),
            ],
            AnyProceduralFieldWithDimensionality(d) => {
                vec![AnyProceduralFieldWithDimensionality(*d)]
            }
            AnyProceduralFieldWithCompositeSize(c) => vec![AnyProceduralFieldWithCompositeSize(*c)],
            AnyFloat123 => vec![AnyFloat123],
            ConcreteType(_) => vec![],
        };

        let mut concrete_domain: Vec<AbstractDataType> = self
            .get_concrete_domain()
            .into_iter()
            .map(|t| ConcreteType(t))
            .collect();
        domain.append(&mut concrete_domain);

        domain
    }
}
