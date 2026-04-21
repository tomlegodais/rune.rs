mod booth;
mod buttons;
mod open;

pub(super) const BANK_COMPONENT: u16 = 87;
pub(super) const INV_COMPONENT: u16 = 0;

pub(super) const BANK_SIZE_VARC: u16 = 192;
pub(super) const FREE_BANK_SIZE_VARC: u16 = 1038;
pub(super) const LAST_X_VARP: u16 = 1249;

pub(super) fn tab_from_component(c: u16) -> Option<u8> {
    ((40..=56).contains(&c) && (c - 40).is_multiple_of(2)).then(|| 8 - ((c - 40) / 2) as u8)
}
