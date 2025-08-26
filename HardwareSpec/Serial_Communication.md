# **Cicada-16 Serial Communication**

The Cicada-16 features a serial communication interface that allows two consoles to be linked together for multiplayer gameplay. The system is designed to be hot-swappable, with a protocol that handles connection, role negotiation, and disconnection gracefully.

## **1. Hardware Registers**

Serial communication is controlled by two I/O registers.

| Address | Name | Description                                                              |
| :------ | :--- | :----------------------------------------------------------------------- |
| F002    | **SB** | **Serial Buffer (R/W)** - Holds one byte of data to be transferred.      |
| F003    | **SC** | **Serial Control (R/W)** - Configures and controls the serial port.      |

### **SC (F003) - Serial Control Register**

| Bit | Name        | Type | Function                                                                                                                   |
| :-- | :---------- | :--- | :------------------------------------------------------------------------------------------------------------------------- |
| 7   | START       | R/W  | **Start Transfer:** Writing a 1 to this bit begins a transfer. It is automatically cleared by hardware when the transfer is complete. |
| 6   | CONNECTED   | R    | **Connection Status:** 1 if a link partner is detected, 0 otherwise. This bit is read-only.                                |
| 5-3 | -           | R    | (Reserved)                                                                                                               |
| 2   | SPEED       | R/W  | **Transfer Speed:** 0 = Normal (8 Kbps), 1 = Fast (256 Kbps).                                                              |
| 1   | CLK_SRC     | R/W  | **Clock Source:** 0 = Master (Internal Clock), 1 = Slave (External Clock).                                               |
| 0   | ENABLE      | R/W  | **Serial Enable:** 1 = Serial port is enabled, 0 = Disabled.                                                               |

The `CONNECTED` bit is set to 1 by the hardware whenever the console's Serial In (SI) pin detects a high logic level, which indicates that another active console is on the other end of the cable.

## **2. Link Status Interrupt**

To allow for efficient, event-driven handling of connection events, a dedicated interrupt is provided.

-   **Interrupt:** Link Status Interrupt
-   **IE/IF Bit:** 5
-   **Trigger:** This interrupt is requested by the hardware whenever the `CONNECTED` status bit in the `SC` register changes its state (from 0 to 1, or from 1 to 0).

This allows the game's CPU to ignore the connection status until the hardware interrupts it, at which point an interrupt service routine can run to handle the "Connected" or "Disconnected" event immediately.

## **3. Hot-Swappable Protocol**

The following protocol is recommended for establishing a stable, hot-swappable link between two consoles.

### **Step 1: Idle State & Presence Detection**

1.  A game enters its multiplayer lobby or menu.
2.  It enables its serial port by setting `SC.ENABLE` to 1.
3.  It sets its Serial Out (SO) pin to a high logic level to signal its readiness. This is typically the default state when the port is enabled.
4.  The game can now either periodically check the `SC.CONNECTED` bit or, more efficiently, wait for the **Link Status Interrupt**.

### **Step 2: Connection & Handshake**

1.  When a player plugs in the link cable, each console's SI pin is connected to the other's high SO pin.
2.  The hardware on both consoles detects this high logic level and automatically sets their `SC.CONNECTED` bit to 1.
3.  If interrupts are used, the **Link Status Interrupt** is triggered on both consoles.
4.  The software on both sides now knows a connection has been made.

### **Step 3: Master/Slave Role Negotiation**

To avoid needing players to choose roles manually, the consoles can negotiate automatically.

1.  Upon connection, each console reads a pseudo-random number from one of the free-running `DIV` registers.
2.  They immediately perform a pre-arranged byte transfer, sending their random number to the other console.
3.  Each console compares the number it sent with the number it received.
4.  The console that had the **higher** number automatically sets its `SC.CLK_SRC` bit to `0` (Master). The other console sets its bit to `1` (Slave).
5.  In the rare event of a tie, the process is simply repeated until a winner is decided.

### **Step 4: Data Transfer**

Once roles are established, the Master console initiates all data transfers by setting the `SC.START` bit. The Slave console must have data ready in its `SB` register.

### **Step 5: Handling Disconnection**

1.  If a player unplugs the cable at any time, the SI pin on the remaining console will go low.
2.  The hardware will immediately and automatically set the `SC.CONNECTED` bit back to 0.
3.  This change triggers the **Link Status Interrupt**.
4.  The game's interrupt handler can then gracefully exit the multiplayer session (e.g., "Your friend has disconnected.") instead of crashing or hanging while waiting for a transfer that will never complete.
