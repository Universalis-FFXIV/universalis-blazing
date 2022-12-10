use ironworks::excel::Excel;
use ironworks_sheets::{for_type, sheet};

pub fn does_world_exist(excel: &Excel, world_id: u32) -> anyhow::Result<bool> {
    let result = excel
        .sheet(for_type::<sheet::World>())
        .and_then(|sheet| sheet.row(world_id))
        .and_then(|row| Ok(row.is_public && row.name.to_string() != "Chaos"))?;
    Ok(result)
}
