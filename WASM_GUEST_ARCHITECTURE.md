# WASM Guest Module Architecture (Pure Message-Passing Model)

This document details the internal architecture for the WASM "guest" modules, defining a pure message-passing contract they must follow to integrate with the Rust host.

---

### 1. Core Principle: Pure, Stateless Execution

The foundational principle of this architecture is that WASM guest modules are **pure, stateless functions**. This means:
- A module's output depends **only** on the data provided in its input messages.
- Modules **cannot** make any external calls back to the host to query for more data.
- The Rust host is responsible for gathering all necessary world-state information and packaging it into the messages *before* dispatching them to a guest.

This design ensures that guest logic is deterministic, highly parallelizable, and completely sandboxed.

---

### 2. WASM Guest ABI (Application Binary Interface)

This is the low-level communication contract between the Rust host and any WASM module.

*   **Exported Functions:** Every WASM module **MUST** export the following C-compatible functions:
    *   `__guest_alloc(size: u32) -> u32`: A function for the host to request the guest to allocate a memory buffer of a given size. Returns a pointer to the allocated block within the guest's linear memory.
    *   `__guest_dealloc(ptr: u32)`: Frees a memory buffer previously allocated by `__guest_alloc`.
    *   `handle_messages(ptr: u32, len: u32) -> u64`: The primary entry point for game logic. It takes a pointer (`ptr`) to a serialized batch of input messages and its length (`len`). It returns a `u64` where the upper 32 bits are the pointer to the serialized output messages and the lower 32 bits are the length.

*   **Data Serialization:** All data passed across the ABI boundary will be serialized using `bincode` for its performance and seamless integration with Rust's `serde` framework.

---

### 3. Internal Guest Logic and Message Handling Workflow

The internal workflow of a guest module is a simple, linear process:

1.  **Entry Point:** The host calls the `handle_messages` function with a pointer to the serialized input data.
2.  **Deserialize:** The guest deserializes the input data into a `Vec<Message>`, which contains all the information needed for processing.
3.  **Process:** The guest iterates through the messages and applies its game logic. Since all data is local, this step involves no external calls.
4.  **Serialize:** The results of the processing (new messages) are collected into an output `Vec<Message>` and serialized into a byte buffer owned by the guest.
5.  **Return:** The guest returns the pointer and length of its output buffer to the host.

---

### 4. Memory Management Between Host and Guest

The architecture uses a "caller allocates, caller frees" pattern to manage memory safely across the WASM boundary.

1.  **Host-to-Guest:** The host allocates a buffer in the guest's memory using `__guest_alloc`, writes the input data, calls `handle_messages`, and then frees the buffer with `__guest_dealloc`.
2.  **Guest-to-Host:** The guest serializes its output, returns the pointer and length, and the host reads the data. The host is then responsible for calling `__guest_dealloc` to allow the guest to free the output buffer.

---

### 5. Handling Complex Scenarios

This pure message-passing architecture is well-suited to handle the complex and performance-sensitive aspects of a Minecraft server.

*   **Redstone Logic:** A `RedstoneUpdate` message would be dispatched to a `Redstone.wasm` module. The host would package the message with the state of the block that changed *and* the state of all its relevant neighbors. The WASM module would then execute the complex Redstone logic and return a batch of `BlockStateChange` messages for any neighbors that were affected, which would be processed in the next tick.

*   **Fast-Moving Players & Physics:** A `PlayerMove` message would contain not only the player's ID and intended new position but also a compact, serialized representation of the nearby world geometry needed for collision detection. The Rust host queries the world state to build this "collision shape" and attaches it to the message. The `Physics.wasm` module can then perform its calculations without needing to ask the host for more data.

*   **Scalability (Multi-core & Multi-server):** This architecture is highly scalable.
    *   **Multi-core:** Since the WASM modules are pure functions with no side effects, the host can safely process messages for different players, different world chunks, or different Redstone circuits in parallel across multiple CPU cores without any risk of data races.
    *   **Multi-server:** The message-passing system can be extended over a network. A central server could distribute workloads (e.g., different dimensions or regions) to other headless server instances, which would run the WASM guests and return the results. The stateless nature of the guests makes this sharding model straightforward to implement.
