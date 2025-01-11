module examples::basic {
    // A simple counter to demonstrate state management
    struct Counter has key {
        value: u64
    }

    // Initialize the module
    fun init() {
    }

    // Increment the counter
    public fun increment(): u64 {
        42 // Simplified for our first example
    }
}