/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Minimal historical `AVPlayerLayer` state contract.
//!
//! This deliberately does not claim that touchHLE can decode or present an
//! AVPlayer stream. It supplies the real CALayer subclass identity and the
//! retained player association used by applications while building their
//! layer tree.

use crate::frameworks::core_animation::ca_layer;
use crate::objc::{id, msg, nil, objc_classes, ClassExports};

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation AVPlayerLayer: CALayer

+ (id)playerLayerWithPlayer:(id)player {
    let layer: id = msg![env; this layer];
    ca_layer::set_media_player(env, layer, player);
    layer
}

- (id)initWithPlayer:(id)player {
    ca_layer::set_media_player(env, this, player);
    this
}

- (id)player {
    ca_layer::media_player(env, this)
}

- (())setPlayer:(id)player {
    ca_layer::set_media_player(env, this, player)
}

@end

};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objc::{msg_class, release};
    use crate::Environment;

    #[test]
    fn player_layer_is_a_calayer_and_round_trips_player_identity() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let player: id = msg_class![env; NSObject new];
        let layer: id = msg_class![env; AVPlayerLayer playerLayerWithPlayer:player];
        assert_ne!(layer, nil);
        let actual: id = msg![env; layer player];
        assert_eq!(actual, player);
        let calayer = env.objc.get_known_class("CALayer", &mut env.mem);
        let layer_class: id = msg![env; layer class];
        assert!(env.objc.class_is_subclass_of(layer_class, calayer));
        release(env, layer);
        release(env, player);
    }
}
