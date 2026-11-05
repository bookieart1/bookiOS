/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `UITableViewController`.
//!
//! The controller owns a `UITableView` as its root view and installs itself as
//! the table's data source and delegate, matching the essential UIKit contract.

use crate::frameworks::core_graphics::CGRect;
use crate::frameworks::foundation::NSInteger;
use crate::impl_HostObject_with_superclass;
use crate::objc::{
    id, msg, msg_class, msg_super, nil, objc_classes, release, ClassExports, NSZonePtr,
};

type UITableViewStyle = NSInteger;

struct UITableViewControllerHostObject {
    superclass: super::UIViewControllerHostObject,
    style: UITableViewStyle,
    clears_selection_on_view_will_appear: bool,
}

fn make_table_view(env: &mut crate::Environment, this: id) -> id {
    let style = env
        .objc
        .borrow::<UITableViewControllerHostObject>(this)
        .style;
    let screen: id = msg_class![env; UIScreen mainScreen];
    let frame: CGRect = msg![env; screen applicationFrame];
    let table: id = msg_class![env; UITableView alloc];
    let table: id = msg![env; table initWithFrame:frame style:style];
    () = msg![env; table setDataSource:this];
    () = msg![env; table setDelegate:this];
    table
}
impl_HostObject_with_superclass!(UITableViewControllerHostObject);
impl Default for UITableViewControllerHostObject {
    fn default() -> Self {
        Self {
            superclass: Default::default(),
            style: 0,
            clears_selection_on_view_will_appear: true,
        }
    }
}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation UITableViewController: UIViewController

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::<UITableViewControllerHostObject>::default();
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

- (id)initWithStyle:(UITableViewStyle)style {
    let this: id = msg_super![env; this initWithNibName:nil bundle:nil];
    if this == nil {
        return nil;
    }

    env.objc.borrow_mut::<UITableViewControllerHostObject>(this).style = style;

    let table = make_table_view(env, this);
    () = msg![env; this setTableView:table];
    release(env, table);
    this
}

- (())loadView {
    // Give an explicitly supplied nib its normal chance to establish the
    // view. If the fallback is not a table, enforce UITableViewController's
    // historical root-view contract rather than leaking a plain UIView.
    () = msg_super![env; this loadView];
    let view: id = msg![env; this tableView];
    let table_class = env.objc.get_known_class("UITableView", &mut env.mem);
    let is_table: bool = msg![env; view isKindOfClass:table_class];
    if !is_table {
        let table = make_table_view(env, this);
        () = msg![env; this setTableView:table];
        release(env, table);
    }
}

- (id)tableView {
    msg![env; this view]
}

- (())setTableView:(id)table_view {
    () = msg![env; this setView:table_view];
}

- (bool)clearsSelectionOnViewWillAppear {
    env.objc
        .borrow::<UITableViewControllerHostObject>(this)
        .clears_selection_on_view_will_appear
}

- (())setClearsSelectionOnViewWillAppear:(bool)clears {
    env.objc
        .borrow_mut::<UITableViewControllerHostObject>(this)
        .clears_selection_on_view_will_appear = clears;
}

@end

};
