use crate::prelude::*;

const AUTOGRAD_NOT_AVAILABLE: &str = "Autograd<> is not available.";

impl<'a, T, D, S> Buffer<'a, T, D, S>
where
    T: Clone + One + 'static,
    D: TapeActions + WriteBuf<T, S, D> + Alloc<T> + 'static,
    S: Shape,
{
    /// Calls `.backward_seeded` on the [`Tape`].
    /// The seed of the gradient is set to `1` and contains `self.len()` elements.
    #[inline]
    #[cfg(feature = "autograd")]
    pub fn backward(&self) {
        if let Some(tape) = unsafe { self.device().tape_mut() } {
            tape.backward_seeded(self)
        }
    }

    /// Returns a reference to the gradient of this buffer.
    /// This allocates a gradient buffer if it wasn't previously. 
    ///
    /// Panics if the gradient was not allocated.
    #[inline]
    #[cfg(feature = "autograd")]
    pub fn grad(&self) -> &'a Self {
        unsafe {
            self.device()
                .tape_mut()
                .expect(AUTOGRAD_NOT_AVAILABLE)
                .grads
                .get_ref(self.device(), self.id())
        }
    }

    /// Returns a reference to the gradient of this buffer.
    /// Returns none either if the autograd feature is disabled, no tape was found (add [`Autograd`] module) or no gradient was allocated previously.
    // TODO: Maybe return Result with two error variants?
    pub fn try_grad(&self) -> Option<&'a Self> {
         unsafe {
            self.device()
                .tape()?
                .grads
                .may_get_ref(self.id()).ok()
        } 
    }

    /// In this case, this is just a dummy function.
    /// Activate the `autograd` feature to make this function useable.
    #[inline]
    #[cfg(not(feature = "autograd"))]
    pub fn grad(&self) -> &'a Self {
        unimplemented!("Gradient not available. Activate the autograd feature.");
    }

    /// Returns a mutable reference to the gradient of this buffer.
    /// This allocates a gradient buffer if it wasn't previously. 
    #[inline]
    #[cfg(feature = "autograd")]
    pub fn grad_mut(&self) -> &'a mut Self {
        unsafe {
            self.device()
                .tape_mut()
                .expect(AUTOGRAD_NOT_AVAILABLE)
                .grads
                .get_mut(self.device(), self.id())
        }
    }
    
    /// Returns a mutable reference to the gradient of this buffer.
    /// Returns none either if the autograd feature is disabled, no tape was found (add [`Autograd`] module) or no gradient was allocated previously.
    // TODO: Maybe return Result with two error variants?
    pub fn try_grad_mut(&self) -> Option<&'a mut Self> {
         unsafe {
            self.device()
                .tape_mut()?
                .grads
                .may_get_mut(self.id()).ok()
        } 
    }


    
    /// In this case, this is just a dummy function.
    /// Activate the `autograd` feature to make this function useable.
    #[inline]
    #[cfg(not(feature = "autograd"))]
    pub fn grad_mut(&self) -> &'a mut Self {
        unimplemented!("Gradient not available. Activate the autograd feature.");
    }
}
