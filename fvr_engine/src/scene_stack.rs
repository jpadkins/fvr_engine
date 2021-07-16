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
    fn load(&mut self, input: &InputManager, terminal: &mut Terminal) -> Result<()>;

    //---------------------------------------------------------------------------------------------
    // Called when the scene is removed from the stack.
    //---------------------------------------------------------------------------------------------
    fn unload(&mut self, input: &InputManager, terminal: &mut Terminal) -> Result<()>;

    //---------------------------------------------------------------------------------------------
    // Called when the scene is made current again (e.g. a the next scene was popped).
    //---------------------------------------------------------------------------------------------
    fn focus(&mut self, input: &InputManager, terminal: &mut Terminal) -> Result<()>;

    //---------------------------------------------------------------------------------------------
    // Called when the scene is made no longer current (e.g. a new scene is pushed).
    //---------------------------------------------------------------------------------------------
    fn unfocus(&mut self, input: &InputManager, terminal: &mut Terminal) -> Result<()>;

    //---------------------------------------------------------------------------------------------
    // Called whenever the scene's (non-visual) internal state should be updated.
    //---------------------------------------------------------------------------------------------
    fn update(
        &mut self,
        dt: &Duration,
        input: &InputManager,
        terminal: &mut Terminal,
    ) -> Result<SceneAction>;

    //---------------------------------------------------------------------------------------------
    // Called whenever the scene's (visual) internal state should be updated and rendered.
    //---------------------------------------------------------------------------------------------
    fn render(&mut self, dt: &Duration, terminal: &mut Terminal) -> Result<()>;
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
    pub fn push(
        &mut self,
        scene: Box<dyn Scene>,
        input: &InputManager,
        terminal: &mut Terminal,
    ) -> Result<()> {
        #[cfg(debug_assertions)]
        println!("[SceneStack] Push - current stack len: {}.", self.scenes.len());

        // Reset the cursor
        input.set_cursor(Cursor::Arrow);

        // Unfocus the current scene if present.
        match self.scenes.last_mut() {
            Some(s) => s.unfocus(input, terminal),
            _ => Ok(()),
        }?;

        // Push the new scene.
        self.scenes.push(scene);

        // Call load on the new scene.
        self.scenes.last_mut().unwrap().load(input, terminal)?;

        Ok(())
    }

    //---------------------------------------------------------------------------------------------
    // Pops the current scene off the stack.
    //---------------------------------------------------------------------------------------------
    pub fn pop(&mut self, input: &InputManager, terminal: &mut Terminal) -> Result<()> {
        #[cfg(debug_assertions)]
        println!("[SceneStack] Pop  - current stack len: {}.", self.scenes.len());

        // Reset the cursor
        input.set_cursor(Cursor::Arrow);

        // Call unload on the current scene.
        self.scenes.last_mut().unwrap().unload(input, terminal)?;

        // Pop the current scene.
        let _ = self.scenes.pop();

        // If a previous scene exists, call focus on it.
        match self.scenes.last_mut() {
            Some(s) => s.focus(input, terminal),
            _ => Ok(()),
        }?;

        Ok(())
    }

    //---------------------------------------------------------------------------------------------
    // Swaps the current scene with a new scene.
    //---------------------------------------------------------------------------------------------
    pub fn swap(
        &mut self,
        scene: Box<dyn Scene>,
        input: &InputManager,
        terminal: &mut Terminal,
    ) -> Result<()> {
        #[cfg(debug_assertions)]
        println!("[SceneStack] Swap - current stack len: {}.", self.scenes.len());

        // Reset the cursor
        input.set_cursor(Cursor::Arrow);

        // Call unload on the current scene.
        self.scenes.last_mut().unwrap().unload(input, terminal)?;

        // Pop the current scene.
        let _ = self.scenes.pop();

        // Push the new scene.
        self.scenes.push(scene);

        // Call load on the new scene.
        self.scenes.last_mut().unwrap().load(input, terminal)?;

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
            SceneAction::Push(scene) => self.push(scene, input, terminal)?,
            SceneAction::Pop => self.pop(input, terminal)?,
            SceneAction::Swap(scene) => self.swap(scene, input, terminal)?,
        }

        // Return false if no scenes exist on the stack.
        Ok(!self.scenes.is_empty())
    }

    //---------------------------------------------------------------------------------------------
    // Renders the current scene.
    //---------------------------------------------------------------------------------------------
    pub fn render(&mut self, dt: &Duration, terminal: &mut Terminal) -> Result<()> {
        self.scenes.last_mut().unwrap().render(dt, terminal)
    }
}
