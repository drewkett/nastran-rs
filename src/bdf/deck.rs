use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::io;

use crate::bdf::{
    parser::{parse_bytes_iter, BulkCard, Field, FieldConv},
    Error, Result,
};

#[derive(Debug, Clone)]
pub struct GRID {
    id: u32,
    cp: u32,
    x: f64,
    y: f64,
    z: f64,
    cd: u32,
    ps: [bool; 6],
    seid: u32,
}

impl GRID {
    fn xyz(&self) -> XYZ {
        XYZ(euclid::Point3D::new(self.x, self.y, self.z))
    }
}

impl TryFrom<BulkCard> for GRID {
    type Error = Error;
    fn try_from(card: BulkCard) -> Result<Self> {
        match card.card_type().as_ref() {
            Some(b"GRID   ") => {}
            Some(c) => return Err(Error::UnexpectedCardType(*b"GRID   ", *c)),
            None => return Err(Error::UnexpectedCardType(*b"GRID   ", *b"       ")),
        }
        let mut iter = card.fields().iter().cloned();
        let id = iter.next().id()?;
        let cp = iter.next().id_or(0)?;
        let x = iter.next().float_or(0.0)?;
        let y = iter.next().float_or(0.0)?;
        let z = iter.next().float_or(0.0)?;
        // is this the right default?
        let cd = iter.next().id_or(0)?;
        let ps = iter.next().dof()?;
        let seid = iter.next().id_or(0)?;
        Ok(GRID {
            id,
            cp,
            x,
            y,
            z,
            cd,
            ps,
            seid,
        })
    }
}

#[derive(Debug, Clone)]
pub struct CORD2R {
    id: u32,
    rid: u32,
    x0: f64,
    y0: f64,
    z0: f64,
    x1: f64,
    y1: f64,
    z1: f64,
    x2: f64,
    y2: f64,
    z2: f64,
}

impl CORD2R {
    fn rotation_matrix(&self) -> RotationMat {
        let Self {
            x0,
            y0,
            z0,
            x1,
            y1,
            z1,
            x2,
            y2,
            z2,
            ..
        } = *self;
        let g0 = euclid::Point3D::<_, U>::new(x0, y0, z0);
        let g1 = euclid::Point3D::<_, U>::new(x1, y1, z1);
        let g2 = euclid::Point3D::<_, U>::new(x2, y2, z2);
        let z = (g1 - g0).normalize();
        let x = g2 - g0;
        let y = z.cross(x).normalize();
        let x = y.cross(z);
        let x = x.to_array();
        let y = y.to_array();
        let z = z.to_array();

        RotationMat(euclid::Transform3D::row_major(
            x[0], x[1], x[2], 0., y[0], y[1], y[2], 0., z[0], z[1], z[2], 0., x0, y0, z0, 1.,
        ))
    }
}

impl TryFrom<BulkCard> for CORD2R {
    type Error = Error;
    fn try_from(card: BulkCard) -> Result<Self> {
        match card.card_type().as_ref() {
            Some(b"CORD2R ") => {}
            Some(c) => return Err(Error::UnexpectedCardType(*b"CORD2R ", *c)),
            None => return Err(Error::UnexpectedCardType(*b"CORD2R ", *b"       ")),
        }
        let mut iter = card.fields().iter().cloned();
        let id = iter.next().id()?;
        let rid = iter.next().id_or(0)?;
        let x0 = iter.next().float_or(0.0)?;
        let y0 = iter.next().float_or(0.0)?;
        let z0 = iter.next().float_or(0.0)?;
        let x1 = iter.next().float_or(0.0)?;
        let y1 = iter.next().float_or(0.0)?;
        let z1 = iter.next().float_or(0.0)?;
        let x2 = iter.next().float_or(0.0)?;
        let y2 = iter.next().float_or(0.0)?;
        let z2 = iter.next().float_or(0.0)?;
        Ok(CORD2R {
            id,
            rid,
            x0,
            y0,
            z0,
            x1,
            y1,
            z1,
            x2,
            y2,
            z2,
        })
    }
}

#[derive(Debug, Clone)]
pub struct CTETRA {
    eid: u32,
    pid: u32,
    g1: u32,
    g2: u32,
    g3: u32,
    g4: u32,
}

impl TryFrom<BulkCard> for CTETRA {
    type Error = Error;
    fn try_from(card: BulkCard) -> Result<Self> {
        match card.card_type().as_ref() {
            Some(b"CTETRA ") => {}
            Some(c) => return Err(Error::UnexpectedCardType(*b"CTETRA ", *c)),
            None => return Err(Error::UnexpectedCardType(*b"CTETRA ", *b"       ")),
        }
        let mut iter = card.fields().iter().cloned();
        let eid = iter.next().id()?;
        let pid = iter.next().id()?;
        let g1 = iter.next().id()?;
        let g2 = iter.next().id()?;
        let g3 = iter.next().id()?;
        let g4 = iter.next().id()?;
        Ok(CTETRA {
            eid,
            pid,
            g1,
            g2,
            g3,
            g4,
        })
    }
}

#[derive(Debug, Clone)]
pub struct PSOLID {
    pid: u32,
    mid: u32,
    cordm: u32,
    r#in: Field,
    stress: Field,
    isop: Field,
    fctn: Field,
}

impl TryFrom<BulkCard> for PSOLID {
    type Error = Error;
    fn try_from(card: BulkCard) -> Result<Self> {
        match card.card_type().as_ref() {
            Some(b"PSOLID ") => {}
            Some(c) => return Err(Error::UnexpectedCardType(*b"PSOLID ", *c)),
            None => return Err(Error::UnexpectedCardType(*b"PSOLID ", *b"       ")),
        }
        let mut iter = card.fields().iter().cloned();
        let pid = iter.next().id()?;
        let mid = iter.next().id()?;
        let cordm = iter.next().id_or(0)?;
        let r#in = iter.next().unwrap_or_default();
        let stress = iter.next().unwrap_or_default();
        let isop = iter.next().unwrap_or_default();
        let fctn = iter.next().unwrap_or_default();
        Ok(PSOLID {
            pid,
            mid,
            cordm,
            r#in,
            stress,
            isop,
            fctn,
        })
    }
}
#[derive(Debug, Clone)]
pub struct MAT1 {
    mid: u32,
    e: f64,
    g: f64,
    nu: f64,
    rho: f64,
    a: f64,
    tref: f64,
    ge: f64,
}

impl TryFrom<BulkCard> for MAT1 {
    type Error = Error;
    fn try_from(card: BulkCard) -> Result<Self> {
        match card.card_type().as_ref() {
            Some(b"MAT1   ") => {}
            Some(c) => return Err(Error::UnexpectedCardType(*b"MAT1   ", *c)),
            None => return Err(Error::UnexpectedCardType(*b"MAT1   ", *b"       ")),
        }
        let mut iter = card.fields().iter().cloned();
        let mid = iter.next().id()?;
        let field_e = iter.next().unwrap_or_default();
        let field_g = iter.next().unwrap_or_default();
        let field_nu = iter.next().unwrap_or_default();
        let e = field_e.maybe_float()?;
        let g = field_g.maybe_float()?;
        let nu = field_nu.maybe_float()?;
        let (e, g, nu) = if e.is_none() && g.is_none() {
            return Err(Error::InvalidMaterialCard(field_e, field_g, field_nu));
        } else if e.is_some() && g.is_some() {
            let e = e.unwrap();
            let g = g.unwrap();
            let nu = nu.unwrap_or(1.0 - e / (2.0 * g));
            (e, g, nu)
        } else if e.is_some() {
            let e = e.unwrap();
            if let Some(nu) = nu {
                let g = e / (2. * (1. + nu));
                (e, g, nu)
            } else {
                (e, 0.0, 0.0)
            }
        } else {
            let g = g.unwrap();
            if let Some(nu) = nu {
                let e = 2. * (1. + nu) * g;
                (e, g, nu)
            } else {
                (0.0, g, 0.0)
            }
        };

        let rho = iter.next().float_or(0.)?;
        let a = iter.next().float_or(0.)?;
        let tref = iter.next().float_or(0.)?;
        let ge = iter.next().float_or(0.)?;
        Ok(MAT1 {
            mid,
            e,
            g,
            nu,
            rho,
            a,
            tref,
            ge,
        })
    }
}

struct U();
pub struct XYZ(euclid::Point3D<f64, U>);

pub struct RotationMat(euclid::Transform3D<f64, U, U>);

impl RotationMat {
    fn forward(&self, xyz: XYZ) -> XYZ {
        XYZ(self.0.transform_point3d(xyz.0).unwrap())
    }

    fn reverse(&self, xyz: XYZ) -> XYZ {
        XYZ(self.0.inverse().unwrap().transform_point3d(xyz.0).unwrap())
    }
}

pub struct GlobalLocation {
    xyz: HashMap<u32, XYZ>,
    csys: HashMap<u32, RotationMat>,
}

#[derive(Debug, Default)]
pub struct Deck {
    grid: Vec<GRID>,
    cord2r: Vec<CORD2R>,
    psolid: Vec<PSOLID>,
    mat1: Vec<MAT1>,
    ctetra: Vec<CTETRA>,
}

impl Deck {
    pub fn from_bytes<I>(iter: I) -> Result<Self>
    where
        I: Iterator<Item = io::Result<u8>>,
    {
        let mut deck: Deck = Default::default();
        let mut iter = parse_bytes_iter(iter);
        while let Some(card) = iter.next() {
            let card = card?;
            // This should be ordered by most common card type. Or maybe using a regexset or something
            match card.card_type().as_ref() {
                Some(b"GRID   ") => deck.grid.push(card.try_into()?),
                Some(b"CORD2R ") => deck.cord2r.push(card.try_into()?),
                Some(b"PSOLID ") => deck.psolid.push(card.try_into()?),
                Some(b"MAT1   ") => deck.mat1.push(card.try_into()?),
                Some(b"CTETRA ") => deck.ctetra.push(card.try_into()?),
                _ => {}
            }
        }
        Ok(deck)
    }

    fn global_locations(&self) -> GlobalLocation {
        let n_grid = self.grid.len();
        let mut xyz = HashMap::with_capacity(n_grid);
        let n_cord = self.cord2r.len();
        let mut csys = HashMap::with_capacity(n_cord);
        let mut grid = self.grid.clone();
        let mut cord2r = self.cord2r.clone();
        grid.retain(|g| {
            if g.cp == 0 {
                xyz.insert(g.id, g.xyz());
                false
            } else {
                true
            }
        });
        cord2r.retain(|c| {
            if c.rid == 0 {
                csys.insert(c.rid, c.rotation_matrix());
                false
            } else {
                true
            }
        });
        while !grid.is_empty() && !cord2r.is_empty() {
            grid.retain(|g| {
                if let Some(r) = csys.get(&g.cp) {
                    xyz.insert(g.id, r.forward(g.xyz()));
                    false
                } else {
                    true
                }
            })
        }
        GlobalLocation { xyz, csys }
    }
}
