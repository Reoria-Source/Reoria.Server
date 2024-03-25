use crate::{
    containers::Storage,
    gametypes::*,
    items::Item,
    tasks::{DataTaskToken, MapItemPacket},
    time_ext::MyInstant,
};
use bytey::{ByteBufferRead, ByteBufferWrite};
use hecs::World;

use super::create_mapitem;

#[derive(Copy, Clone, PartialEq, Eq, Default, ByteBufferRead, ByteBufferWrite)]
pub struct MapItem {
    pub item: Item,
    #[bytey(skip)]
    pub despawn: Option<MyInstant>,
    #[bytey(skip)]
    pub ownertimer: Option<MyInstant>,
    #[bytey(skip)]
    pub ownerid: Option<Entity>,
    pub pos: Position,
}

impl MapItem {
    #[inline(always)]
    pub fn new(num: u32) -> Self {
        let mut item = MapItem::default();
        item.item.num = num;
        item
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Default)]
pub struct DropItem {
    pub index: u32,
    pub amount: u16,
    pub pos: Position,
}

pub fn find_drop_pos(
    world: &mut World,
    storage: &Storage,
    drop_item: DropItem,
) -> Vec<(Position, Option<Entity>)> {
    let mut result = Vec::new();

    let storage_mapitem = storage.map_items.borrow_mut();
    let item_base = match storage.bases.items.get(drop_item.index as usize) {
        Some(data) => data,
        None => return result,
    };

    let mut got_slot = false;
    if !storage_mapitem.contains_key(&drop_item.pos) {
        let mapdata = storage.maps.get(&drop_item.pos.map);
        if let Some(map_data) = mapdata {
            if !map_data
                .borrow()
                .is_blocked_tile(drop_item.pos, WorldEntityType::MapItem)
            {
                result.push((drop_item.pos, None));
                got_slot = true;
            }
        }
    } else {
        'endcheck: for x in drop_item.pos.x - 1..=drop_item.pos.x + 1 {
            for y in drop_item.pos.y - 1..=drop_item.pos.y + 1 {
                let mut check_pos = Position {
                    x,
                    y,
                    ..drop_item.pos
                };
                if check_pos.x < 0 {
                    check_pos.x = 31;
                    check_pos.map.x -= 1;
                }
                if check_pos.x >= 32 {
                    check_pos.x = 0;
                    check_pos.map.x += 1;
                }
                if check_pos.y < 0 {
                    check_pos.y = 31;
                    check_pos.map.y -= 1;
                }
                if check_pos.y >= 32 {
                    check_pos.y = 0;
                    check_pos.map.y += 1;
                }

                if !storage_mapitem.contains_key(&check_pos) {
                    let mapdata = storage.maps.get(&check_pos.map);
                    if let Some(map_data) = mapdata {
                        if !map_data
                            .borrow()
                            .is_blocked_tile(check_pos, WorldEntityType::MapItem)
                        {
                            result.push((check_pos, None));
                            got_slot = true;
                            break 'endcheck;
                        }
                    }
                }
            }
        }
    }

    if !got_slot && item_base.stackable {
        let mut leftover = drop_item.amount;

        'endcheck: for x in drop_item.pos.x - 1..=drop_item.pos.x + 1 {
            for y in drop_item.pos.y - 1..=drop_item.pos.y + 1 {
                let mut check_pos = Position {
                    x,
                    y,
                    ..drop_item.pos
                };
                if check_pos.x < 0 {
                    check_pos.x = 31;
                    check_pos.map.x -= 1;
                }
                if check_pos.x >= 32 {
                    check_pos.x = 0;
                    check_pos.map.x += 1;
                }
                if check_pos.y < 0 {
                    check_pos.y = 31;
                    check_pos.map.y -= 1;
                }
                if check_pos.y >= 32 {
                    check_pos.y = 0;
                    check_pos.map.y += 1;
                }

                if let Some(entity) = storage_mapitem.get(&check_pos) {
                    let mapitem = world.cloned_get_or_panic::<MapItem>(entity);
                    if mapitem.item.num == drop_item.index
                        && mapitem.item.val < item_base.stacklimit
                    {
                        let remaining_val = item_base.stacklimit - mapitem.item.val;
                        leftover = leftover.saturating_sub(remaining_val);
                        result.push((check_pos, Some(*entity)));

                        if leftover == 0 {
                            break 'endcheck;
                        }
                    }
                }
            }
        }
    }

    result
}

pub fn try_drop_item(
    world: &mut World,
    storage: &Storage,
    drop_item: DropItem,
    despawn: Option<MyInstant>,
    ownertimer: Option<MyInstant>,
    ownerid: Option<Entity>,
) -> bool {
    let item_base = match storage.bases.items.get(drop_item.index as usize) {
        Some(data) => data,
        None => return false,
    };

    // Find open position
    let set_pos = find_drop_pos(world, storage, drop_item);
    if set_pos.is_empty() {
        return false;
    }

    let mut leftover = drop_item.amount;
    for found_pos in set_pos.iter() {
        if item_base.stackable
            && let Some(got_entity) = found_pos.1
        {
            if let Ok(mut mapitem) = world.get::<&mut MapItem>(got_entity.0) {
                mapitem.item.val = mapitem.item.val.saturating_add(leftover);
                if mapitem.item.val > item_base.stacklimit {
                    leftover = mapitem.item.val - item_base.stacklimit;
                    mapitem.item.val = item_base.stacklimit;
                } else {
                    break;
                }
            }
        } else {
            let mut storage_mapitem = storage.map_items.borrow_mut();
            let mapdata = storage.maps.get(&found_pos.0.map);
            if let Some(map_data) = mapdata {
                let mut map_item = create_mapitem(drop_item.index, drop_item.amount, found_pos.0);
                map_item.despawn = despawn;
                map_item.ownertimer = ownertimer;
                map_item.ownerid = ownerid;
                let id = world.spawn((WorldEntityType::MapItem, map_item));
                let _ = world.insert_one(id, EntityType::MapItem(Entity(id)));
                map_data.borrow_mut().itemids.insert(Entity(id));
                storage_mapitem.insert(found_pos.0, Entity(id));
                let _ = DataTaskToken::ItemLoad(found_pos.0.map).add_task(
                    storage,
                    &MapItemPacket::new(
                        Entity(id),
                        map_item.pos,
                        map_item.item,
                        map_item.ownerid,
                        true,
                    ),
                );
            }
            break;
        }
    }

    true
}
