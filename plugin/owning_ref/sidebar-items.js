initSidebarItems({"struct":[["OwningRef","An owning reference."]],"trait":[["CloneStableAddress","Marker trait for expressing that the memory address of the value reachable via a dereference remains identical even if `self` is a clone."],["Erased","Helper trait for an erased concrete type an owner dereferences to. This is used in form of a trait object for keeping something around to (virtually) call the destructor."],["IntoErased","Helper trait for erasing the concrete type of what an owner derferences to, for example `Box<T> -> Box<Erased>`. This would be unneeded with higher kinded types support in the language."],["StableAddress","Marker trait for expressing that the memory address of the value reachable via a dereference remains identical even if `self` gets moved."]],"type":[["ArcRef","Typedef of a owning reference that uses a `Arc` as the owner."],["BoxRef","Typedef of a owning reference that uses a `Box` as the owner."],["ErasedArcRef","Typedef of a owning reference that uses an erased `Arc` as the owner."],["ErasedBoxRef","Typedef of a owning reference that uses an erased `Box` as the owner."],["ErasedRcRef","Typedef of a owning reference that uses an erased `Rc` as the owner."],["MutexGuardRef","Typedef of a owning reference that uses a `MutexGuard` as the owner."],["RcRef","Typedef of a owning reference that uses a `Rc` as the owner."],["RefMutRef","Typedef of a owning reference that uses a `RefMut` as the owner."],["RefRef","Typedef of a owning reference that uses a `Ref` as the owner."],["RwLockReadGuardRef","Typedef of a owning reference that uses a `RwLockReadGuard` as the owner."],["RwLockWriteGuardRef","Typedef of a owning reference that uses a `RwLockWriteGuard` as the owner."],["StringRef","Typedef of a owning reference that uses a `String` as the owner."],["VecRef","Typedef of a owning reference that uses a `Vec` as the owner."]]});