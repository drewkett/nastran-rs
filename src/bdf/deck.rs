use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};

use crate::bdf::{
    parser::{parse_bytes, BulkCard, Field, FieldConv},
    Error, Result,
};
use crate::util::{CoordSys, Vec3, XYZ};

#[derive(Debug, Clone)]
pub struct GRID {
    id: u32,
    cp: u32,
    xyz: XYZ,
    cd: u32,
    ps: [bool; 6],
    seid: u32,
}

impl StorageItem for GRID {
    type Id = u32;

    fn id(&self) -> Self::Id {
        self.id
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
        let xyz = XYZ::new(x, y, z);
        // is this the right default?
        let cd = iter.next().id_or(0)?;
        let ps = iter.next().dof()?;
        let seid = iter.next().id_or(0)?;
        Ok(GRID {
            id,
            cp,
            xyz,
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
    fn rotation_matrix(&self) -> CoordSys {
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
        let g0 = Vec3::new(x0, y0, z0);
        let g1 = Vec3::new(x1, y1, z1);
        let g2 = Vec3::new(x2, y2, z2);
        let z = (g1 - g0).normalize();
        let x = g2 - g0;
        let y = z.cross(x).normalize();
        let x = y.cross(z);

        CoordSys::new(x, y, z, g0)
    }
}

impl StorageItem for CORD2R {
    type Id = u32;

    fn id(&self) -> Self::Id {
        self.id
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

impl CTETRA {
    fn volume(&self, location: &GlobalLocation) -> Option<f64> {
        let xyz1 = location.get_grid(self.g1)?.to_vec();
        let xyz2 = location.get_grid(self.g2)?.to_vec();
        let xyz3 = location.get_grid(self.g3)?.to_vec();
        let xyz4 = location.get_grid(self.g4)?.to_vec();
        let volume = (xyz2 - xyz1).cross(xyz3 - xyz1).dot(xyz4 - xyz1) / 6.;
        Some(volume)
    }
    fn volume_cg(&self, location: &GlobalLocation) -> Option<(f64, Vec3)> {
        let xyz1 = location.get_grid(self.g1)?.to_vec();
        let xyz2 = location.get_grid(self.g2)?.to_vec();
        let xyz3 = location.get_grid(self.g3)?.to_vec();
        let xyz4 = location.get_grid(self.g4)?.to_vec();
        let volume = (xyz2 - xyz1).cross(xyz3 - xyz1).dot(xyz4 - xyz1) / 6.;
        let cg = (xyz1 + xyz2 + xyz3 + xyz4) / 4.;
        Some((volume, cg))
    }
}

#[derive(Default)]
struct MassMoment {
    mass: f64,
    moment: Vec3,
}

impl std::ops::Add<MassMoment> for MassMoment {
    type Output = MassMoment;
    fn add(self, rhs: MassMoment) -> MassMoment {
        let mass = self.mass + rhs.mass;
        let moment = self.moment + rhs.moment;
        MassMoment { mass, moment }
    }
}

impl std::ops::AddAssign<MassMoment> for MassMoment {
    fn add_assign(&mut self, rhs: MassMoment) {
        self.mass += rhs.mass;
        self.moment += rhs.moment;
    }
}

impl std::iter::Sum for MassMoment {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Default::default(), |i, m| i + m)
    }
}

impl DeckRef<'_, CTETRA> {
    fn mass(&self, location: &GlobalLocation) -> Option<f64> {
        let density = self.density()?;
        let volume = self.volume(location)?;
        Some(density * volume)
    }
    fn mass_cg(&self, location: &GlobalLocation) -> Option<(f64, Vec3)> {
        let density = self.density()?;
        let (volume, cg) = self.volume_cg(location)?;
        Some((density * volume, cg))
    }
    fn mass_moment(&self, location: &GlobalLocation) -> Option<MassMoment> {
        let (mass, cg) = self.mass_cg(location)?;
        let moment = mass * cg;
        Some(MassMoment { mass, moment })
    }
}

impl StorageItem for CTETRA {
    type Id = u32;

    fn id(&self) -> Self::Id {
        self.eid
    }
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

impl StorageItem for PSOLID {
    type Id = u32;

    fn id(&self) -> Self::Id {
        self.pid
    }
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

impl MAT1 {
    #[allow(dead_code)]
    fn density(&self) -> f64 {
        self.rho
    }
}

impl StorageItem for MAT1 {
    type Id = u32;

    fn id(&self) -> Self::Id {
        self.mid
    }
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
        } else if let Some(e) = e {
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

pub trait StorageItem: Clone {
    type Id: std::hash::Hash + Eq;
    fn id(&self) -> Self::Id;
}
#[derive(Debug)]
pub struct Storage<T>
where
    T: StorageItem,
{
    data: Vec<Option<T>>,
    map: HashMap<T::Id, usize>,
}

impl<T> From<RawStorage<T>> for Storage<T>
where
    T: StorageItem,
{
    fn from(raw: RawStorage<T>) -> Self {
        // FIXME This doesn't currently check for duplicates
        let data = raw.data;
        let map = data
            .iter()
            .enumerate()
            .filter_map(|(i, v)| {
                if let Some(v) = v {
                    Some((v.id(), i))
                } else {
                    None
                }
            })
            .collect();
        Storage { data, map }
    }
}

impl<T> Storage<T>
where
    T: StorageItem,
{
    #[allow(dead_code)]
    fn new() -> Self {
        Self {
            data: Vec::new(),
            map: HashMap::new(),
        }
    }

    fn with_capacity(n: usize) -> Self {
        Self {
            data: Vec::with_capacity(n),
            map: HashMap::with_capacity(n),
        }
    }

    fn len(&self) -> usize {
        self.map.len()
    }

    fn get(&self, id: T::Id) -> Option<&T> {
        self.map.get(&id).and_then(|i| self.data[*i].as_ref())
    }

    #[cfg_attr(feature = "parallel", allow(dead_code))]
    fn replace(&mut self, item: T) -> Option<T> {
        let i = self.data.len();
        let id = item.id();
        self.data.push(Some(item));
        match self.map.insert(id, i) {
            Some(i) => self.data[i].take(),
            None => None,
        }
    }

    #[cfg_attr(feature = "parallel", allow(dead_code))]
    fn insert(&mut self, item: T) -> Result<()> {
        match self.replace(item) {
            Some(_item) => Err(Error::DuplicateCard),
            None => Ok(()),
        }
    }

    #[cfg_attr(not(feature = "parallel"), allow(dead_code))]
    fn extend_from_raw(&mut self, raw: RawStorage<T>) -> Result<()> {
        let n = self.data.len();
        self.data.extend(raw.data);
        for (i, item) in self.data[n..].iter().enumerate() {
            match self.map.insert(item.as_ref().unwrap().id(), i + n) {
                Some(_i) => return Err(Error::DuplicateCard),
                None => {}
            }
        }
        Ok(())
    }

    fn clone_to_vec(&self) -> Vec<T> {
        self.data
            .iter()
            .filter_map(|i| i.as_ref())
            .cloned()
            .collect()
    }

    fn iter(&self) -> impl Iterator<Item = &T> {
        self.data.iter().filter_map(|c| c.as_ref())
    }
}

impl<T> Default for Storage<T>
where
    T: StorageItem,
{
    fn default() -> Self {
        Self {
            data: Vec::new(),
            map: HashMap::new(),
        }
    }
}

pub struct GlobalLocation {
    xyz: HashMap<u32, XYZ>,
    csys: HashMap<u32, CoordSys>,
}

impl GlobalLocation {
    pub fn get_grid(&self, id: u32) -> Option<XYZ> {
        self.xyz.get(&id).copied()
    }

    pub fn get_csys(&self, id: u32) -> Option<&CoordSys> {
        self.csys.get(&id)
    }
}

#[derive(Clone)]
struct DeckRef<'a, T> {
    deck: &'a Deck,
    item: &'a T,
}

impl<'a, T> std::ops::Deref for DeckRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.item
    }
}

trait HasProperty<'a> {
    type Property;
    fn property(&'a self) -> Option<DeckRef<'a, Self::Property>>;
}

impl<'a> HasProperty<'a> for DeckRef<'a, CTETRA> {
    type Property = PSOLID;

    fn property(&'a self) -> Option<DeckRef<'a, Self::Property>> {
        self.deck.psolid(self.item.pid)
    }
}

trait HasDensity {
    fn density(&self) -> f64;
}

impl HasDensity for MAT1 {
    fn density(&self) -> f64 {
        self.rho
    }
}

trait HasMaterial<'a> {
    type Material: HasDensity;

    fn material(&'a self) -> Option<DeckRef<'a, Self::Material>>;

    fn density(&'a self) -> Option<f64> {
        self.material().map(|r| r.density())
    }
}

impl<'a> HasMaterial<'a> for DeckRef<'a, PSOLID> {
    type Material = MAT1;

    fn material(&self) -> Option<DeckRef<Self::Material>> {
        self.deck.mat1(self.item.mid)
    }
}

impl<'a> HasMaterial<'a> for DeckRef<'a, CTETRA> {
    type Material = MAT1;

    fn material(&self) -> Option<DeckRef<Self::Material>> {
        self.property().and_then(|p| self.deck.mat1(p.id()))
    }
}

#[derive(Debug)]
pub struct RawStorage<T> {
    data: Vec<Option<T>>,
}

impl<T> RawStorage<T> {
    #[allow(dead_code)]
    fn insert(&mut self, value: T) {
        self.data.push(Some(value))
    }
}

impl<T> Default for RawStorage<T> {
    fn default() -> Self {
        Self { data: Vec::new() }
    }
}

#[derive(Debug, Default)]
pub struct DeckCounts {
    grid: usize,
    cord2r: usize,
    psolid: usize,
    mat1: usize,
    ctetra: usize,
}

#[derive(Debug, Default)]
pub struct RawDeck {
    grid: RawStorage<GRID>,
    cord2r: RawStorage<CORD2R>,
    psolid: RawStorage<PSOLID>,
    mat1: RawStorage<MAT1>,
    ctetra: RawStorage<CTETRA>,
}

impl Into<Deck> for RawDeck {
    fn into(self) -> Deck {
        Deck {
            grid: self.grid.into(),
            cord2r: self.cord2r.into(),
            psolid: self.psolid.into(),
            mat1: self.mat1.into(),
            ctetra: self.ctetra.into(),
        }
    }
}

#[derive(Debug, Default)]
pub struct Deck {
    grid: Storage<GRID>,
    cord2r: Storage<CORD2R>,
    psolid: Storage<PSOLID>,
    mat1: Storage<MAT1>,
    ctetra: Storage<CTETRA>,
}

impl Deck {
    #[cfg(feature = "parallel")]
    pub fn from_filename(filename: impl AsRef<std::path::Path>) -> Result<Self> {
        use rayon::prelude::*;
        let bytes = std::fs::read(filename)?;
        let cards: Vec<_> = parse_bytes(bytes)?.collect();
        let decks = cards
            .into_par_iter()
            .try_fold(RawDeck::default, |mut deck, card| -> Result<RawDeck> {
                // This should be ordered by most common card type. Or maybe using a regexset or something
                let card = card?;
                match card.card_type().as_ref() {
                    Some(b"GRID   ") => deck.grid.insert(card.try_into()?),
                    Some(b"CORD2R ") => deck.cord2r.insert(card.try_into()?),
                    Some(b"PSOLID ") => deck.psolid.insert(card.try_into()?),
                    Some(b"MAT1   ") => deck.mat1.insert(card.try_into()?),
                    Some(b"CTETRA ") => deck.ctetra.insert(card.try_into()?),
                    _ => {}
                }
                Ok(deck)
            })
            .collect::<Result<Vec<_>>>()?;
        let counts = decks
            .iter()
            .fold(DeckCounts::default(), |mut counts, item| {
                counts.grid += item.grid.data.len();
                counts.cord2r += item.cord2r.data.len();
                counts.psolid += item.psolid.data.len();
                counts.mat1 += item.mat1.data.len();
                counts.ctetra += item.ctetra.data.len();
                counts
            });

        decks
            .into_iter()
            .try_fold(Deck::with_capacity(counts), |mut deck, item| {
                deck.grid.extend_from_raw(item.grid)?;
                deck.cord2r.extend_from_raw(item.cord2r)?;
                deck.psolid.extend_from_raw(item.psolid)?;
                deck.mat1.extend_from_raw(item.mat1)?;
                deck.ctetra.extend_from_raw(item.ctetra)?;
                Ok(deck)
            })
    }

    pub fn with_capacity(counts: DeckCounts) -> Self {
        Self {
            grid: Storage::with_capacity(counts.grid),
            cord2r: Storage::with_capacity(counts.cord2r),
            psolid: Storage::with_capacity(counts.psolid),
            mat1: Storage::with_capacity(counts.mat1),
            ctetra: Storage::with_capacity(counts.ctetra),
        }
    }

    #[cfg(not(feature = "parallel"))]
    pub fn from_filename(filename: impl AsRef<std::path::Path>) -> Result<Self> {
        let bytes = std::fs::read(filename)?;
        parse_bytes(bytes)?.try_fold(Deck::default(), |mut deck, card| {
            // This should be ordered by most common card type. Or maybe using a regexset or something
            let card = card?;
            match card.card_type().as_ref() {
                Some(b"GRID   ") => deck.grid.insert(card.try_into()?)?,
                Some(b"CORD2R ") => deck.cord2r.insert(card.try_into()?)?,
                Some(b"PSOLID ") => deck.psolid.insert(card.try_into()?)?,
                Some(b"MAT1   ") => deck.mat1.insert(card.try_into()?)?,
                Some(b"CTETRA ") => deck.ctetra.insert(card.try_into()?)?,
                _ => {}
            };
            Ok(deck)
        })
    }

    pub fn global_locations(&self) -> GlobalLocation {
        let t = std::time::Instant::now();
        let n_grid = self.grid.len();
        let mut xyz = HashMap::with_capacity(n_grid);
        let n_cord = self.cord2r.len();
        let mut csys = HashMap::with_capacity(n_cord);
        let mut grid = self.grid.clone_to_vec();
        let mut cord2r = self.cord2r.clone_to_vec();
        grid.retain(|g| {
            if g.cp == 0 {
                xyz.insert(g.id, g.xyz);
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
                    xyz.insert(g.id, r.forward(g.xyz));
                    false
                } else {
                    true
                }
            })
        }
        println!("global_locations took {} ms", t.elapsed().as_millis());
        GlobalLocation { xyz, csys }
    }

    fn with<'a, T>(&'a self, item: &'a T) -> DeckRef<'a, T> {
        DeckRef { deck: self, item }
    }

    #[allow(dead_code)]
    fn grid(&self, id: u32) -> Option<DeckRef<GRID>> {
        self.grid.get(id).map(|grid| self.with(grid))
    }

    #[allow(dead_code)]
    fn tetra(&self, id: u32) -> Option<DeckRef<CTETRA>> {
        self.ctetra.get(id).map(|e| self.with(e))
    }

    fn psolid(&self, id: u32) -> Option<DeckRef<PSOLID>> {
        self.psolid.get(id).map(|e| self.with(e))
    }

    fn mat1(&self, id: u32) -> Option<DeckRef<MAT1>> {
        self.mat1.get(id).map(|e| self.with(e))
    }

    pub fn mass(&self, location: &GlobalLocation) -> f64 {
        self.ctetra
            .iter()
            .map(|c| self.with(c).mass(location).unwrap_or_default())
            .sum()
    }

    #[cfg(feature = "parallel")]
    pub fn mass_cg(&self, location: &GlobalLocation) -> (f64, Vec3) {
        use rayon::prelude::*;
        let mm: MassMoment = self
            .ctetra
            .data
            .par_iter()
            .filter_map(|c| c.as_ref())
            .map(|c| self.with(c).mass_moment(location).unwrap_or_default())
            .sum();
        let cg = mm.moment / mm.mass;
        (mm.mass, cg)
    }

    #[cfg(not(feature = "parallel"))]
    pub fn mass_cg(&self, location: &GlobalLocation) -> (f64, Vec3) {
        let mm: MassMoment = self
            .ctetra
            .data
            .iter()
            .filter_map(|c| c.as_ref())
            .map(|c| self.with(c).mass_moment(location).unwrap_or_default())
            .sum();
        let cg = mm.moment / mm.mass;
        (mm.mass, cg)
    }
}
