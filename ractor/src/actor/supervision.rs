// Copyright (c) Sean Lawlor
//
// This source code is licensed under both the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree.

//! Supervision management logic

use std::sync::Arc;

use dashmap::DashMap;

use super::{actor_cell::ActorCell, messages::SupervisionEvent};
use crate::{ActorHandler, ActorId};

/// A supervision tree
#[derive(Clone, Default)]
pub struct SupervisionTree {
    children: Arc<DashMap<ActorId, ActorCell>>,
    parents: Arc<DashMap<ActorId, ActorCell>>,
}

impl SupervisionTree {
    /// Push a child into the tere
    pub fn insert_child(&self, child: ActorCell) {
        self.children.insert(child.get_id(), child);
    }

    /// Remove a specific actor from the supervision tree (e.g. actor died)
    pub fn remove_child(&self, child: ActorCell) {
        let id = child.get_id();
        match self.children.entry(id) {
            dashmap::mapref::entry::Entry::Occupied(item) => {
                item.remove();
            }
            dashmap::mapref::entry::Entry::Vacant(_) => {}
        }
    }

    /// Push a parent into the tere
    pub fn insert_parent(&self, parent: ActorCell) {
        self.parents.insert(parent.get_id(), parent);
    }

    /// Remove a specific actor from the supervision tree (e.g. actor died)
    pub fn remove_parent(&self, parent: ActorCell) {
        let id = parent.get_id();
        match self.parents.entry(id) {
            dashmap::mapref::entry::Entry::Occupied(item) => {
                item.remove();
            }
            dashmap::mapref::entry::Entry::Vacant(_) => {}
        }
    }

    /// Terminate all your supervised children
    pub fn terminate_children(&self) {
        for kvp in self.children.iter() {
            kvp.value().terminate();
        }
    }

    /// Determine if the specified actor is a member of this supervision tree
    pub fn is_supervisor_of(&self, id: ActorId) -> bool {
        self.children.contains_key(&id)
    }

    /// Determine if the specified actor is a parent of this actor
    pub fn is_child_of(&self, id: ActorId) -> bool {
        self.parents.contains_key(&id)
    }

    /// Send a notification to all supervisors
    pub fn notify_supervisors<TActor, TState>(&self, evt: SupervisionEvent)
    where
        TActor: ActorHandler<State = TState>,
        TState: crate::State,
    {
        for kvp in self.parents.iter() {
            let evt_clone = evt.duplicate::<TState>().unwrap();
            let _ = kvp.value().send_supervisor_evt(evt_clone);
        }
    }

    /// Retrieve the number of supervised children
    pub fn get_num_children(&self) -> usize {
        self.children.len()
    }

    /// Retrieve the number of supervised children
    pub fn get_num_parents(&self) -> usize {
        self.parents.len()
    }
}