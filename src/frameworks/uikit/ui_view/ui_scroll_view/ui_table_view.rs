/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `UITableView`.
//!
//! This models the historical data-source-driven table contract used by early
//! iPhone applications. Cell reuse and decorations remain intentionally basic.

use crate::frameworks::core_graphics::{CGFloat, CGPoint, CGRect, CGSize};
use crate::frameworks::foundation::NSInteger;
use crate::objc::{
    autorelease, id, impl_HostObject_with_superclass, msg, msg_class, msg_super, nil,
    objc_classes, release, retain, todo_objc_setter, ClassExports, NSZonePtr, SEL,
};

type UITableViewStyle = NSInteger;
const UITableViewStylePlain: UITableViewStyle = 0;
type UITableViewCellSeparatorStyle = NSInteger;
type UITableViewScrollPosition = NSInteger;
const UITableViewScrollPositionNone: UITableViewScrollPosition = 0;
const UITableViewScrollPositionTop: UITableViewScrollPosition = 1;
const UITableViewScrollPositionMiddle: UITableViewScrollPosition = 2;
const UITableViewScrollPositionBottom: UITableViewScrollPosition = 3;

fn index_path_for_flat_row(
    section_row_counts: &[NSInteger],
    mut flat_row: NSInteger,
) -> Option<(NSInteger, NSInteger)> {
    if flat_row < 0 {
        return None;
    }
    for (section, row_count) in section_row_counts.iter().copied().enumerate() {
        if flat_row < row_count {
            return Some((section as NSInteger, flat_row));
        }
        flat_row -= row_count;
    }
    None
}

pub struct UITableViewHostObject {
    superclass: super::UIScrollViewHostObject,
    /// UITableViewDataSource, weak reference
    data_source: id,
    style: UITableViewStyle,
    row_height: CGFloat,
    section_row_counts: Vec<NSInteger>,
    allows_selection: bool,
    visible_cells: Vec<id>,
    /// Cells retained by the table after leaving the visible hierarchy.
    reusable_cells: Vec<id>,
}
impl_HostObject_with_superclass!(UITableViewHostObject);
impl Default for UITableViewHostObject {
    fn default() -> Self {
        UITableViewHostObject {
            superclass: Default::default(),
            data_source: nil,
            style: UITableViewStylePlain,
            row_height: 44.0,
            section_row_counts: Vec::new(),
            allows_selection: true,
            visible_cells: Vec::new(),
            reusable_cells: Vec::new(),
        }
    }
}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation UITableView: UIScrollView

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::<UITableViewHostObject>::default();
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

- (id)dataSource {
    env.objc.borrow::<UITableViewHostObject>(this).data_source
}
- (())setDataSource:(id)data_source {
    env.objc.borrow_mut::<UITableViewHostObject>(this).data_source = data_source;
}

- (id)initWithFrame:(CGRect)frame style:(UITableViewStyle)style {
    let this: id = msg![env; this initWithFrame:frame];
    env.objc.borrow_mut::<UITableViewHostObject>(this).style = style;
    this
}
- (UITableViewStyle)style {
    env.objc.borrow::<UITableViewHostObject>(this).style
}

- (CGFloat)rowHeight {
    env.objc.borrow::<UITableViewHostObject>(this).row_height
}
- (())setRowHeight:(CGFloat)row_height {
    env.objc.borrow_mut::<UITableViewHostObject>(this).row_height = row_height;
}

// Nothing is drawn yet, so these do nothing.
- (())setSeparatorStyle:(UITableViewCellSeparatorStyle)style {
    todo_objc_setter!(this, style);
}
- (())setSeparatorColor:(id)color {
    todo_objc_setter!(this, color);
}
- (())setSectionHeaderHeight:(CGFloat)height {
    todo_objc_setter!(this, height);
}
- (())setSectionFooterHeight:(CGFloat)height {
    todo_objc_setter!(this, height);
}
- (())setAllowsSelection:(bool)allows {
    env.objc.borrow_mut::<UITableViewHostObject>(this).allows_selection = allows;
}
- (bool)allowsSelection {
    env.objc.borrow::<UITableViewHostObject>(this).allows_selection
}
- (())setTableHeaderView:(id)view {
    todo_objc_setter!(this, view);
}
- (())setTableFooterView:(id)view {
    todo_objc_setter!(this, view);
}
- (())setBackgroundView:(id)view {
    todo_objc_setter!(this, view);
}
- (())reloadData {
    env.diagnostic_selector_event("table-reload-begin", this, nil, "reloadData", nil, None);
    let old_cells = std::mem::take(
        &mut env.objc.borrow_mut::<UITableViewHostObject>(this).visible_cells,
    );
    for cell in old_cells {
        let reuse_identifier: id = msg![env; cell reuseIdentifier];
        if reuse_identifier != nil {
            // Transfer the subview's ownership into the bounded reuse pool
            // before removeFromSuperview releases the hierarchy's retain.
            retain(env, cell);
        }
        () = msg![env; cell removeFromSuperview];
        if reuse_identifier != nil {
            env.objc
                .borrow_mut::<UITableViewHostObject>(this)
                .reusable_cells
                .push(cell);
        }
    }
    let data_source = env.objc.borrow::<UITableViewHostObject>(this).data_source;
    let mut section_row_counts = Vec::new();
    if data_source != nil {
        let sections_selector = env.objc.register_host_selector(
            "numberOfSectionsInTableView:".to_string(),
            &mut env.mem,
        );
        let provides_sections: bool = msg![env; data_source respondsToSelector:sections_selector];
        let section_count: NSInteger = if provides_sections {
            msg![env; data_source numberOfSectionsInTableView:this]
        } else {
            1
        };
        assert!(
            section_count >= 0,
            "UITableView data source returned a negative section count"
        );
        for section in 0..section_count {
            let row_count: NSInteger =
                msg![env; data_source tableView:this numberOfRowsInSection:section];
            assert!(
                row_count >= 0,
                "UITableView data source returned a negative row count"
            );
            section_row_counts.push(row_count);
        }
    }

    let row_height = env.objc.borrow::<UITableViewHostObject>(this).row_height;
    let total_rows: NSInteger = section_row_counts.iter().copied().sum();
    let bounds: CGRect = msg![env; this bounds];
    let size = CGSize {
        width: bounds.size.width,
        height: total_rows as CGFloat * row_height,
    };
    () = msg![env; this setContentSize:size];

    // Older applications often construct their entire short menu table in one
    // pass. Ask the data source for those cells and install them as ordinary
    // subviews. Bound the eager implementation so a very large data set cannot
    // turn this partial contract into unbounded work; later cell reuse can lift
    // this limit without changing the data-source semantics.
    let mut cells = Vec::new();
    let mut flat_row: NSInteger = 0;
    const MAX_EAGER_CELLS: usize = 512;
    'sections: for (section, row_count) in section_row_counts.iter().copied().enumerate() {
        let section = section as NSInteger;
        for row in 0..row_count {
            if cells.len() == MAX_EAGER_CELLS {
                log!("UITableView {:?}: bounded eager cell limit reached", this);
                break 'sections;
            }
            let index_path: id =
                msg_class![env; NSIndexPath indexPathForRow:row inSection:section];
            let cell: id = msg![env; data_source tableView:this cellForRowAtIndexPath:index_path];
            assert_ne!(cell, nil, "UITableView data source returned a nil cell");
            let frame = CGRect {
                origin: CGPoint { x: 0.0, y: flat_row as CGFloat * row_height },
                size: CGSize { width: bounds.size.width, height: row_height },
            };
            () = msg![env; cell setFrame:frame];
            () = msg![env; this addSubview:cell];
            cells.push(cell);
            flat_row += 1;
        }
    }
    {
        let host = env.objc.borrow_mut::<UITableViewHostObject>(this);
        host.section_row_counts = section_row_counts;
        host.visible_cells = cells;
    }
    env.diagnostic_selector_event(
        "table-reload-end", this, data_source, "reloadData", nil, Some(total_rows as f64)
    );
    log_dbg!(
        "[(UITableView*) {:?} reloadData] laid out {} uniform rows",
        this,
        total_rows
    );
}

- (id)dequeueReusableCellWithIdentifier:(id)identifier {
    assert_ne!(identifier, nil, "UITableView reuse identifier must not be nil");
    let reusable_cells = env
        .objc
        .borrow::<UITableViewHostObject>(this)
        .reusable_cells
        .clone();
    let mut matching_index = None;
    for (index, cell) in reusable_cells.into_iter().enumerate() {
        let candidate: id = msg![env; cell reuseIdentifier];
        let matches: bool = candidate != nil && msg![env; candidate isEqual:identifier];
        if matches {
            matching_index = Some(index);
            break;
        }
    }
    let Some(index) = matching_index else {
        return nil;
    };
    let cell = env
        .objc
        .borrow_mut::<UITableViewHostObject>(this)
        .reusable_cells
        .remove(index);
    () = msg![env; cell prepareForReuse];
    autorelease(env, cell)
}

- (id)cellForRowAtIndexPath:(id)index_path {
    if index_path == nil {
        return nil;
    }
    let section: NSInteger = msg![env; index_path section];
    let row: NSInteger = msg![env; index_path row];
    if section < 0 || row < 0 {
        return nil;
    }
    let host = env.objc.borrow::<UITableViewHostObject>(this);
    let Some(&row_count) = host.section_row_counts.get(section as usize) else {
        return nil;
    };
    if row >= row_count {
        return nil;
    }
    let preceding_rows: NSInteger = host.section_row_counts[..section as usize]
        .iter()
        .copied()
        .sum();
    // This partial table implementation eagerly materializes up to a bounded
    // number of cells. Native UITableView returns nil when a valid row is not
    // currently visible/materialized.
    host.visible_cells
        .get((preceding_rows + row) as usize)
        .copied()
        .unwrap_or(nil)
}

- (())dealloc {
    let reusable_cells = std::mem::take(
        &mut env.objc.borrow_mut::<UITableViewHostObject>(this).reusable_cells,
    );
    for cell in reusable_cells {
        release(env, cell);
    }
    msg_super![env; this dealloc]
}

- (())scrollToRowAtIndexPath:(id)index_path
            atScrollPosition:(UITableViewScrollPosition)position
                    animated:(bool)_animated {
    assert_ne!(index_path, nil, "UITableView cannot scroll to a nil index path");
    let section: NSInteger = msg![env; index_path section];
    let row: NSInteger = msg![env; index_path row];
    let (row_height, content_height, row_start, current_offset) = {
        let host = env.objc.borrow::<UITableViewHostObject>(this);
        assert!(
            section >= 0 && (section as usize) < host.section_row_counts.len(),
            "UITableView section is out of range"
        );
        let row_count = host.section_row_counts[section as usize];
        assert!(
            row >= 0 && row < row_count,
            "UITableView row is out of range"
        );
        let preceding_rows: NSInteger = host.section_row_counts[..section as usize]
            .iter()
            .copied()
            .sum();
        let row_height = host.row_height;
        let content_height = host
            .section_row_counts
            .iter()
            .copied()
            .sum::<NSInteger>() as CGFloat
            * row_height;
        let row_start = (preceding_rows + row) as CGFloat * row_height;
        (
            row_height,
            content_height,
            row_start,
            host.superclass.content_offset,
        )
    };

    let bounds: CGRect = msg![env; this bounds];
    let target = match position {
        UITableViewScrollPositionNone => {
            if row_start < current_offset.y {
                row_start
            } else if row_start + row_height > current_offset.y + bounds.size.height {
                row_start + row_height - bounds.size.height
            } else {
                current_offset.y
            }
        }
        UITableViewScrollPositionTop => row_start,
        UITableViewScrollPositionMiddle => row_start - (bounds.size.height - row_height) / 2.0,
        UITableViewScrollPositionBottom => row_start - (bounds.size.height - row_height),
        _ => panic!("Unknown UITableViewScrollPosition {position}"),
    };
    let max_offset = (content_height - bounds.size.height).max(0.0);
    let target = target.max(0.0).min(max_offset);
    if target != current_offset.y {
        let offset = CGPoint {
            x: current_offset.x,
            y: target,
        };
        () = msg![env; this setContentOffset:offset];
    }
}
- (())deselectRowAtIndexPath:(id)_index_path animated:(bool)_animated {
    // Selection highlighting is not rendered yet.
}

- (())touchesEnded:(id)touches withEvent:(id)_event {
    let (allows_selection, was_dragging, offset, counts, row_height) = {
        let host = env.objc.borrow::<UITableViewHostObject>(this);
        (
            host.allows_selection,
            host.superclass.dragging,
            host.superclass.content_offset,
            host.section_row_counts.clone(),
            host.row_height,
        )
    };
    env.objc.borrow_mut::<UITableViewHostObject>(this).superclass.dragging = false;
    if !allows_selection || was_dragging || counts.is_empty() {
        return;
    }
    let touch: id = msg![env; touches anyObject];
    let point: CGPoint = msg![env; touch locationInView:this];
    let flat = ((point.y + offset.y) / row_height).floor() as NSInteger;
    let selected = index_path_for_flat_row(&counts, flat);
    let Some((section, row)) = selected else {
        env.diagnostic_selector_event("table-selection-miss", this, nil, "touchesEnded:", touch, None);
        return;
    };
    let index_path: id = msg_class![env; NSIndexPath indexPathForRow:row inSection:section];
    let delegate: id = msg![env; this delegate];
    if delegate != nil {
        let selector: SEL = env.objc.register_host_selector(
            "tableView:didSelectRowAtIndexPath:".to_string(), &mut env.mem,
        );
        let responds: bool = msg![env; delegate respondsToSelector:selector];
        if responds {
            env.diagnostic_selector_event(
                "table-selection", this, delegate,
                "tableView:didSelectRowAtIndexPath:", index_path, None
            );
            () = msg![env; delegate tableView:this didSelectRowAtIndexPath:index_path];
        }
    }
}

@end

};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frameworks::foundation::ns_string;
    use crate::Environment;

    #[test]
    fn maps_flat_rows_across_sections_and_rejects_out_of_range_rows() {
        let rows = [2, 0, 3];
        assert_eq!(index_path_for_flat_row(&rows, -1), None);
        assert_eq!(index_path_for_flat_row(&rows, 0), Some((0, 0)));
        assert_eq!(index_path_for_flat_row(&rows, 1), Some((0, 1)));
        assert_eq!(index_path_for_flat_row(&rows, 2), Some((2, 0)));
        assert_eq!(index_path_for_flat_row(&rows, 4), Some((2, 2)));
        assert_eq!(index_path_for_flat_row(&rows, 5), None);
    }

    #[test]
    fn looks_up_materialized_cell_by_section_and_row() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let stack_size = 0x1000;
        let stack = env.mem.alloc(stack_size);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = stack.to_bits() + stack_size;
        let table: id = msg_class![env; UITableView new];
        let first: id = msg_class![env; UITableViewCell new];
        let second: id = msg_class![env; UITableViewCell new];
        retain(env, first);
        retain(env, second);
        {
            let host = env.objc.borrow_mut::<UITableViewHostObject>(table);
            host.section_row_counts = vec![1, 1];
            host.visible_cells = vec![first, second];
        }
        let index_path: id = msg_class![env; NSIndexPath indexPathForRow:0i32 inSection:1i32];
        let found: id = msg![env; table cellForRowAtIndexPath:index_path];
        assert_eq!(found, second);
        release(env, table);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = 0;
        env.mem.free(stack);
    }

    #[test]
    fn dequeue_matches_identifier_and_transfers_queue_ownership() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        // UIView initialization passes CGRect values through the guest ABI.
        // The lightweight fixture has no stack unless the test supplies one.
        let stack_size = 0x1000;
        let stack = env.mem.alloc(stack_size);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = stack.to_bits() + stack_size;
        let pool: id = msg_class![env; NSAutoreleasePool new];
        let table: id = msg_class![env; UITableView new];
        let reuse = ns_string::get_static_str(env, "Cell");
        let other = ns_string::get_static_str(env, "Other");
        let cell: id = msg_class![env; UITableViewCell alloc];
        let cell: id = msg![env; cell initWithStyle:0i32 reuseIdentifier:reuse];
        retain(env, cell);
        env.objc
            .borrow_mut::<UITableViewHostObject>(table)
            .reusable_cells
            .push(cell);
        release(env, cell);
        let missing: id = msg![env; table dequeueReusableCellWithIdentifier:other];
        assert_eq!(missing, nil);
        let returned: id = msg![env; table dequeueReusableCellWithIdentifier:reuse];
        assert_eq!(returned, cell);
        assert!(env
            .objc
            .borrow::<UITableViewHostObject>(table)
            .reusable_cells
            .is_empty());
        release(env, table);
        release(env, pool);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = 0;
        env.mem.free(stack);
    }
}
