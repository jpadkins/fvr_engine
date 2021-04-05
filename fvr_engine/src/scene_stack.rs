//-------------------------------------------------------------------------------------------------
// STD includes.
//-------------------------------------------------------------------------------------------------
use std::fmt::{Display, Formatter};
use std::time::Duration;

//-------------------------------------------------------------------------------------------------
// Extern crate includes.
//-------------------------------------------------------------------------------------------------
use anyhow::Result;

//-------------------------------------------------------------------------------------------------
// Workspace includes.
//-------------------------------------------------------------------------------------------------
use fvr_engine_client::prelude::*;

//-------------------------------------------------------------------------------------------------
// Scene action enumerates the possible actions a scene can return when being updated.
//-------------------------------------------------------------------------------------------------
pub enum SceneAction {
    // The scene stack should do nothing.
    Noop,
    // The scene stack should push a new scene onto the stack.
    Push(Box<dyn Scene>),
    // The scene stack should pop the current scene from the stack.
    Pop,
    // The scene stack should swap the current scene with a new scene.
    Swap(Box<dyn Scene>),
}

impl Display for SceneAction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            SceneAction::Noop => write!(f, "SceneAction::Noop"),
            SceneAction::Push(_) => write!(f, "SceneAction::Push"),
            SceneAction::Pop => write!(f, "SceneAction::Pop"),
            SceneAction::Swap(_) => write!(f, "SceneAction::Swap"),
        }
    }
}

//-------------------------------------------------------------------------------------------------
// Describes the interface for anything that is handled by the scene stack.
//-------------------------------------------------------------------------------------------------
pub trait Scene {
    //---------------------------------------------------------------------------------------------
    // Called when the scene is added to the stack.
    //---------------------------------------------------------------------------------------------
    fn load(&mut self, terminal: &mut Terminal) -> Result<()>;

    //---------------------------------------------------------------------------------------------
    // Called when the scene is removed from the stack.
    //---------------------------------------------------------------------------------------------
    fn unload(&mut self, terminal: &mut Terminal) -> Result<()>;

    //---------------------------------------------------------------------------------------------
    // Called when the scene is made current again (e.g. a the next scene was popped).
    //---------------------------------------------------------------------------------------------
    fn focus(&mut self, terminal: &mut Terminal) -> Result<()>;

    //---------------------------------------------------------------------------------------------
    // Called when the scene is made no longer current (e.g. a new scene is pushed).
    //---------------------------------------------------------------------------------------------
    fn unfocus(&mut self, terminal: &mut Terminal) -> Result<()>;

    //---------------------------------------------------------------------------------------------
    // Called whenever the scene's internal state should be updated and rendered.
    //---------------------------------------------------------------------------------------------
    fn update(
        &mut self,
        dt: &Duration,
        input: &InputManager,
        terminal: &mut Terminal,
    ) -> Result<SceneAction>;
}

//-------------------------------------------------------------------------------------------------
// Scene stack describes a stack of scene objects.
//-------------------------------------------------------------------------------------------------
pub struct SceneStack {
    // The stack of scenes.
    scenes: Vec<Box<dyn Scene>>,
}

impl SceneStack {
    //---------------------------------------------------------------------------------------------
    // Creates a new scene stack.
    // (there should only ever be one)
    //---------------------------------------------------------------------------------------------
    pub fn new() -> Self {
        Self { scenes: Default::default() }
    }

    //---------------------------------------------------------------------------------------------
    // Pushes a new scene onto the stack.
    //---------------------------------------------------------------------------------------------
    pub fn push(&mut self, scene: Box<dyn Scene>, terminal: &mut Terminal) -> Result<()> {
        #[cfg(debug_assertions)]
        println!("[SceneStack] Push - current stack len: {}.", self.scenes.len());

        // Unfocus the current scene if present.
        match self.scenes.last_mut() {
            Some(s) => s.unfocus(terminal),
            _ => Ok(()),
        }?;

        // Push the new scene.
        self.scenes.push(scene);

        // Call load on the new scene.
        self.scenes.last_mut().unwrap().load(terminal)?;

        Ok(())
    }

    //---------------------------------------------------------------------------------------------
    // Pops the current scene off the stack.
    //---------------------------------------------------------------------------------------------
    pub fn pop(&mut self, terminal: &mut Terminal) -> Result<()> {
        #[cfg(debug_assertions)]
        println!("[SceneStack] Pop  - current stack len: {}.", self.scenes.len());

        // Call unload on the current scene.
        self.scenes.last_mut().unwrap().unload(terminal)?;

        // Pop the current scene.
        let _ = self.scenes.pop();

        // If a previous scene exists, call focus on it.
        match self.scenes.last_mut() {
            Some(s) => s.focus(terminal),
            _ => Ok(()),
        }?;

        Ok(())
    }

    //---------------------------------------------------------------------------------------------
    // Swaps the current scene with a new scene.
    //---------------------------------------------------------------------------------------------
    pub fn swap(&mut self, scene: Box<dyn Scene>, terminal: &mut Terminal) -> Result<()> {
        #[cfg(debug_assertions)]
        println!("[SceneStack] Swap - current stack len: {}.", self.scenes.len());

        // Call unload on the current scene.
        self.scenes.last_mut().unwrap().unload(terminal)?;

        // Pop the current scene.
        let _ = self.scenes.pop();

        // Push the new scene.
        self.scenes.push(scene);

        // Call load on the new scene.
        self.scenes.last_mut().unwrap().load(terminal)?;

        Ok(())
    }

    //---------------------------------------------------------------------------------------------
    // Updates the scene stack, also updating the current scene and handling any actions.
    // (returns whether there are any scenes left in the stack)
    //---------------------------------------------------------------------------------------------
    pub fn update(
        &mut self,
        dt: &Duration,
        input: &InputManager,
        terminal: &mut Terminal,
    ) -> Result<bool> {
        // Return false if no scenes exist on the stack.
        if self.scenes.is_empty() {
            return Ok(false);
        }

        // Update the current scene and handle the returned scene action.
        match self.scenes.last_mut().unwrap().update(dt, input, terminal)? {
            SceneAction::Noop => {}
            SceneAction::Push(scene) => self.push(scene, terminal)?,
            SceneAction::Pop => self.pop(terminal)?,
            SceneAction::Swap(scene) => self.swap(scene, terminal)?,
        }

        // Return false if no scenes exist on the stack.
        Ok(!self.scenes.is_empty())
    }
}
