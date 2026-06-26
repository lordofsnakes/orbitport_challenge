# Solution

The response schema for each endpoint was quite similar. I first created private structs corresponding to the return type of the Helios endpoints within the plugin. Essentially the idea behind this is to allow the plugin to be modular for different ground stations providers who may return data with different labelings. After reading each response I deserialize them into structs before mapping it to the corresponding values declared in the types file. As for error handling, I simply looked at the return code from the Helios endpoints, and checked them against documented http response codes, defaulting to the provider error messages where implemented. The shortcuts I took mostly had to do with features that go beyond the requirement specifications, however I did end up spending more than 2 hours on the task as I really wanted to complete it.  The biggest shortcut I took was using Codex to help me with Rust syntax.

A few things that I did not have to handle with the Helios mock (that real ground station providers may want to implement) were:

- Legal regulations,
- Priority Booking
- Removing stale Authentication
- Billing and Refunding

I think one idea that would be quite cool is to implement x402 into Orbitport. Since ground-station contacts and payload downloads are paid, Orbitport could support x402 payments for usage based operations like booking a contact or downloading payload data. That would let developers pay through one standard HTTP-native flow while Orbitport handles provider-specific billing and access control behind the scenes.
