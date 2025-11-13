# WASM Guest Module Architecture

This document details the internal architecture for the WASM "guest" modules, defining the contracts and patterns they must follow to integrate with the Rust host in the Minecraft server reimplementation.

---

### 1. WASM Guest ABI (Application Binary Interface)

This is the foundational contract between the Rust host and any WASM module.

*   **Exported Functions:** Every WASM module **MUST** export the following C-compatible functions:
    *   `__guest_alloc(size: u32) -> u32`: A function for the host to request the guest to allocate a memory buffer of a given size. Returns a pointer to the allocated block within the guest's linear memory.
    *   `__guest_dealloc(ptr: u32)`: Frees a memory buffer previously allocated by `__guest_alloc`.
    *   `handle_messages(ptr: u32, len: u32) -> u64`: The primary entry point for game logic. It takes a pointer (`ptr`) to a serialized batch of input messages and its length (`len`). It returns a `u64` where the upper 32 bits are the pointer to the serialized output messages and the lower 32 bits are the length.

*   **Data Serialization:** All data passed across the ABI boundary (the message batches) will be serialized using `bincode`. This ensures a compact, fast, and language-agnostic representation.

---

### 2. Host Function Interface (Imports for the Guest)

These are the functions the Rust host provides, which the WASM guest can import and call to query the game state.

*   **Namespace:** All host functions will be exposed under a single namespace, e.g., `env`.
*   **Function Signatures (from Guest's perspective):**
    *   `get_block(x: i32, y: i32, z: i32) -> u16`: Returns the ID of the block at a given world coordinate.
    *   `get_entity_data(entity_id: u64, data_ptr: u32, max_len: u32) -> u32`: Fills a guest-allocated buffer (`data_ptr`) with the serialized data for a specific entity. Returns the number of bytes written.
    *   `find_entities_in_radius(center_x: f32, center_y: f32, center_z: f32, radius: f32, data_ptr: u32, max_len: u32) -> u32`: Finds all entities within a given radius and writes their IDs to a guest-allocated buffer. Returns the number of entities found.
    *   `log_message(level: u32, ptr: u32, len: u32)`: Allows the guest to write log messages to the host's logging system.

---

### 3. Internal Guest Logic and Message Handling Workflow

*   **Entry Point:** The `handle_messages` function deserializes the input message batch, processes each message through internal logic, collects the output messages, and returns a new serialized batch.
*   **Statelessness:** The guest module must not contain any persistent state across calls. All necessary information is either in the input message or retrieved via host calls.

---

### 4. Memory Management Between Host and Guest

The architecture uses a "caller allocates, caller frees" pattern to manage memory safely across the WASM boundary.

1.  **Host-to-Guest:** The host allocates a buffer in the guest's memory using `__guest_alloc`, writes the input data, calls `handle_messages`, and then frees the buffer with `__guest_dealloc`.
2.  **Guest-to-Host:** The guest serializes its output, returns the pointer and length, and the host reads the data. The host is then responsible for calling `__guest_dealloc` to allow the guest to free the output buffer.
