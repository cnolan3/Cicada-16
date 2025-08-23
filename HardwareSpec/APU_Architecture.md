# **Cricket-16 APU - Architecture**

This document describes the design of the Audio Processing Unit (APU) for the Cricket-16 fantasy console. The APU is responsible for generating all sound and music. The design is inspired by classic sound chips from the 8-bit and 16-bit eras, featuring four independent sound channels with ADSR envelope control and a simple DSP for effects.

## **1. Core Features**

- **4 Independent Channels**: The APU can play four sounds simultaneously.
- **Channel 0 (Pulse A)**: A square wave generator with variable duty cycle and a hardware frequency sweep unit.
- **Channel 1 (Pulse B)**: A second square wave generator, identical to Pulse A but without the sweep unit.
- **Channel 2 (Wave)**: A flexible waveform generator that plays back a custom 32-sample waveform from Wave RAM.
- **Channel 3 (Noise)**: A noise generator capable of producing various "colors" of noise for percussion and SFX.
- **ADSR Envelopes**: The Pulse, Wave, and Noise channels each feature a unified, hardware-based 4-stage ADSR volume envelope.
- **DSP (Digital Signal Processor)**: A simple hardware unit for creating an echo/delay effect.
- **Stereo Panning**: Each channel's output can be independently panned.
- **Master Volume Control**: Global volume control for both left and right stereo outputs.

## **2. Channel 0 & 1: Pulse Channels**

The two pulse channels generate simple square waves. They are controlled by a set of registers that define their pitch, volume, and timbre.

- **Frequency Control**: A 16-bit frequency register allows for a wide range of musical notes.
- **Volume Envelope**: A hardware ADSR envelope generator for shaping the volume of notes.
- **Duty Cycle**: The "width" of the square wave can be changed (12.5%, 25%, 50%, 75%) to alter the timbre.
- **Frequency Sweep (Channel 0 Only)**: A hardware unit that can automatically slide the frequency of Channel 0 up or down over time.

### 2.1. Frequency and Pitch

The pitch of the two pulse channels is determined by the 16-bit value in their respective `CHx_FREQ` registers. The APU uses a master clock of **4.194304 MHz**. The output frequency is calculated with the following formula:

**`Output Frequency (Hz) = 4,194,304 / (64 * (65536 - FREQ_REG))`**

Where `FREQ_REG` is the 16-bit value from the `F5x4-F5x5` register. This formula allows for fine-grained control over the pitch, covering a wide range of musical notes.

**Example Calculation:**

To produce the note A-4 (440 Hz), a programmer would need to calculate the required `FREQ_REG` value:

1.  `440 = 4,194,304 / (64 * (65536 - FREQ_REG))`
2.  `64 * (65536 - FREQ_REG) = 4,194,304 / 440`
3.  `65536 - FREQ_REG = 9532.5 / 64`
4.  `65536 - FREQ_REG = 148.945...`
5.  `FREQ_REG = 65536 - 149` (rounding to the nearest integer)
6.  `FREQ_REG = 65387`

A value of **65387** in the frequency register will produce a tone very close to 440 Hz. Game developers would typically use a pre-calculated lookup table for note frequencies. The official System Library provides a standardized lookup table for this purpose. See the `System_Library.md` document for more details.

### 2.2. Hardware Frequency Sweep (Channel 0)

Channel 0 includes a hardware frequency sweep unit that can be used to create sliding pitch effects, such as arpeggios, explosions, and risers. The sweep unit periodically recalculates the channel's frequency based on its current frequency and the settings in the `CH0_SWP` register.

The sweep unit is clocked at **128Hz**. The timing of the sweep is controlled by the `SWP_TIME` setting. When the sweep is enabled, the following calculation is performed at each step:

**`New Frequency = Current Frequency +/- (Current Frequency >> SWP_SHIFT)`**

The direction of the sweep (addition or subtraction) is controlled by the `SWP_DIR` bit. If the new calculated frequency goes out of the valid range (0-65535), the channel is disabled.

The sweep unit is controlled by the `CH0_SWP` register.

## **3. Channel 2: Wave Channel**

The wave channel offers the most sonic flexibility by playing back a small, user-defined waveform. This is ideal for creating unique instrument tones or simple speech/sound effects.

- **Wave RAM:** A 4-Kilobyte area of memory at `E000-EFFF` is dedicated to waveform data. This RAM is organized as a bank of 128 unique waveforms, indexed from 0 to 127.
- **Waveform Format:** Each waveform is 32 bytes long and contains 64 sequential 4-bit samples. The samples are packed two per byte, with the high nibble (bits 7-4) being the first sample and the low nibble (bits 3-0) being the second. The 4-bit sample values represent amplitude, from 0 (lowest point) to 15 (highest point).
- **Playback:** When the channel is triggered via `KEY_ON`, the APU begins reading the 64 samples of the selected waveform sequentially. The playback speed is controlled by the `CH2_FREQ` register. When the 64th sample is played, the playback loops back to the first sample. The output of the wave channel is the current sample's 4-bit value, scaled by the channel's current 4-bit ADSR envelope volume.
- **Volume Envelope:** This channel includes the same hardware ADSR envelope found on the Pulse and Noise channels.

### 3.1. Waveform Frequency

The frequency of the `CH2_FREQ` register determines how quickly the APU steps through the 64 samples of the selected waveform. The resulting audible pitch is determined by this playback rate. The formula is intentionally aligned with the pulse channels, allowing them to share the same frequency lookup table.

**`Sample Rate (Hz) = 4,194,304 / (65536 - FREQ_REG)`**

**`Output Pitch (Hz) = Sample Rate / 64`**

This simplifies to the final formula:

**`Output Pitch (Hz) = 4,194,304 / (64 * (65536 - FREQ_REG))`**

Where `FREQ_REG` is the 16-bit value from the `F524-F525` register.

**Note:** Because this formula is identical to the one for the pulse channels, the same note frequency values from the System Library's lookup table can be used for Channel 2.

**Example Calculation:**

To produce the note A-4 (440 Hz), the `FREQ_REG` value is the same as for the pulse channels:

`FREQ_REG = 65387`

This value will cause the 64 samples of the waveform to be played back at a rate that produces a fundamental pitch of 440 Hz.

## **4. Channel 3: Noise Channel**

The noise channel generates pseudo-random noise, which is essential for creating percussive sounds (drums, cymbals) and sound effects (explosions, static). Its sound is generated by a Linear Feedback Shift Register (LFSR) that is clocked at a variable rate.

### 4.1. LFSR Implementation

The LFSR is a shift register whose input bit is derived from the exclusive-OR (XOR) of two bits in its own state. At each clock tick, all bits are shifted one position to the right, and the new input bit becomes the most significant bit (MSB). The output of the channel is determined by the least significant bit (LSB).

The `LFSR_MODE` bit in the `CH3_CTRL` register selects between two different LFSR configurations, which produce different timbres:

-   **Mode 0 (15-bit LFSR):**
    -   Creates a full, dense "white noise" sound.
    -   The new input bit is the result of `Bit 14 XOR Bit 15`.
-   **Mode 1 (7-bit LFSR):**
    -   Creates a metallic, periodic sound because the sequence of pseudo-random numbers is much shorter.
    -   The new input bit is the result of `Bit 6 XOR Bit 7`.

The channel's audio output is `1` if the LSB of the LFSR is `0`, and `-1` if the LSB is `1`. This output is then scaled by the channel's current ADSR envelope volume.

### 4.2. Noise Frequency

The "pitch" of the noise is controlled by the rate at which the LFSR is clocked. This clock rate is derived from the master APU clock and the 6-bit `CLK_DIV` value in the `CH3_CTRL` register.

**`LFSR Clock Rate (Hz) = 4,194,304 / (256 * (CLK_DIV + 1))`**

-   A lower `CLK_DIV` value results in a higher-pitched, "brighter" noise.
-   A higher `CLK_DIV` value results in a lower-pitched, "rumbling" noise.

## **5. ADSR Volume Envelopes**

The APU uses a traditional 4-stage ADSR (Attack, Decay, Sustain, Release) envelope to control channel volume. The envelope's volume level is a 4-bit value (0-15), which modulates the channel's output. The envelope's state machine is updated by a global **Envelope Clock** running at **256Hz**.

When a channel's `KEY_ON` bit is set to 1, the envelope enters the **Attack** phase, where the volume rises from 0 to its peak (15). When it reaches the peak, it moves to the **Decay** phase, where it falls to the specified **Sustain** level. It then remains in the **Sustain** phase as long as `KEY_ON` is 1. When `KEY_ON` is cleared to 0, it immediately enters the **Release** phase, where the volume falls from the current level to 0.

## **6. DSP (Digital Signal Processor)**

The APU includes a simple DSP unit for a configurable echo/delay effect.

- **Functionality**: The DSP captures the final mixed audio output, delays it, and then mixes it back into the main output.
- **Delay Buffer**: Uses a dedicated 1 KiB of RAM at F800-FBFF.
- **Control**: Controlled via registers starting at F5A0 for delay time, feedback, and wet/dry mix.

## **7. APU Registers (F500–F5FF)**

This section details the memory-mapped registers used to control all APU functions, starting with a high-level overview.

### **Register Map Overview**

| Address   | Name      | Description                                                |
| :-------- | :-------- | :--------------------------------------------------------- |
| F500      | CH0_CTRL  | Pulse A: Control Register                                  |
| F501-F502 | CH0_ADSR  | ADSR settings for Channel 0                                |
| F503      | CH0_SWP   | Sweep settings for Channel 0                               |
| F504-F505 | CH0_FREQ  | 16-bit frequency for Channel 0                             |
| F510      | CH1_CTRL  | Pulse B: Control Register                                  |
| F511-F512 | CH1_ADSR  | ADSR settings for Channel 1                                |
| F514-F515 | CH1_FREQ  | 16-bit frequency for Channel 1                             |
| F520      | CH2_CTRL  | Wave channel: Control Register                             |
| F521-F522 | CH2_ADSR  | ADSR settings for Channel 2                                |
| F524-F525 | CH2_FREQ  | 16-bit frequency for Channel 2                             |
| F530      | CH3_CTRL  | Noise channel: Control Register                            |
| F531-F532 | CH3_ADSR  | ADSR settings for Channel 3                                |
| F580      | MIX_CTRL  | Master APU enable, individual channel enables              |
| F581      | MIX_VOL_L | Master volume Left (0-15)                                  |
| F582      | MIX_VOL_R | Master volume Right (0-15)                                 |
| F584-F587 | PAN0-PAN3 | Per-channel stereo panning control                         |
| F5A0      | DSP_CTRL  | DSP Control Register                                       |
| F5A1      | DSP_DELAY | Delay time/length (controls read offset into delay buffer) |
| F5A2      | DSP_FBACK | Feedback level (0-15). Controls echo decay.                |
| F5A3      | DSP_WET   | Wet signal mix level (0-15). Controls echo volume.         |

### **Channel Register Details**

#### **F500: CH0_CTRL (Pulse A)**

| Bit | Name   | Description                                                                                             |
| :-- | :----- | :------------------------------------------------------------------------------------------------------ |
| 7   | KEY_ON | 1 = Note On (Attack->Decay->Sustain). 0 = Note Off (Release). The channel is active when this bit is 1. |
| 6-5 | DUTY   | Square wave duty cycle: 00: 12.5%, 01: 25%, 10: 50%, 11: 75%                                            |
| 4-0 | -      | Unused.                                                                                                 |

#### **F503: CH0_SWP (Pulse A Sweep)**

| Bit | Name      | Description                                                                                             |
| :-- | :-------- | :------------------------------------------------------------------------------------------------------ |
| 7   | SWP_EN    | 1 = Enable the frequency sweep unit. The sweep is triggered when a note is keyed on.                  |
| 6-4 | SWP_TIME  | Time between sweep steps, in units of 7.8ms (1/128s). If 0, sweep is disabled. (Range: 0-7)            |
| 3   | SWP_DIR   | Sweep direction. 0: Addition (frequency increases), 1: Subtraction (frequency decreases).             |
| 2-0 | SWP_SHIFT | Sweep magnitude. The new frequency is calculated by shifting the current frequency this many bits to the right and adding/subtracting it. If 0, sweep has no effect. |

#### **F504-F505: CH0_FREQ**

This 16-bit register controls the frequency of the pulse wave for Channel 0. See section "2.1. Frequency and Pitch" for the calculation formula.

#### **F510: CH1_CTRL (Pulse B)**

| Bit | Name   | Description                                                                                             |
| :-- | :----- | :------------------------------------------------------------------------------------------------------ |
| 7   | KEY_ON | 1 = Note On (Attack->Decay->Sustain). 0 = Note Off (Release). The channel is active when this bit is 1. |
| 6-5 | DUTY   | Square wave duty cycle: 00: 12.5%, 01: 25%, 10: 50%, 11: 75%                                            |
| 4-0 | -      | Unused.                                                                                                 |

#### **F514-F515: CH1_FREQ**

This 16-bit register controls the frequency of the pulse wave for Channel 1. See section "2.1. Frequency and Pitch" for the calculation formula.

#### **F520: CH2_CTRL (Wave)**

| Bit | Name     | Description                                                                                             |
| :-- | :------- | :------------------------------------------------------------------------------------------------------ |
| 7   | KEY_ON   | 1 = Note On (Attack->Decay->Sustain). 0 = Note Off (Release). The channel is active when this bit is 1. |
| 6-0 | WAVE_IDX | Selects which 32-byte waveform to play from Wave RAM (0-127).                                           |

#### **F524-F525: CH2_FREQ**

This 16-bit register controls the playback frequency of the selected waveform for Channel 2. See section "3.1. Waveform Frequency" for the calculation formula.

#### **F530: CH3_CTRL (Noise)**

| Bit | Name      | Description                                                                                             |
| :-- | :-------- | :------------------------------------------------------------------------------------------------------ |
| 7   | KEY_ON    | 1 = Note On (Attack->Decay->Sustain). 0 = Note Off (Release). The channel is active when this bit is 1. |
| 6   | LFSR_MODE | 0: 15-bit LFSR (Tonal noise), 1: 7-bit LFSR (Metallic noise).                                           |
| 5-0 | CLK_DIV   | Clock divider for the LFSR, controls the base pitch of the noise.                                       |

#### **F5x1 / F5x2: ADSR Registers**

These two registers are identical for each channel (CH0, CH1, CH2, CH3).

- **F5x1, Bits 7-4: Attack Rate (A)**
  - This 4-bit value (0-15) determines how quickly the volume rises from 0 to its peak of 15.
  - The value `A` specifies the number of Envelope Clock ticks to wait before the volume is incremented by 1.
  - **Time per step:** `(A + 1) * (1/256 seconds)`
  - **Total Attack Time (0 to 15):** `15 * (A + 1) * ~3.9ms`

- **F5x1, Bits 3-0: Decay Rate (D)**
  - This 4-bit value (0-15) determines how quickly the volume falls from its peak of 15 to the Sustain Level.
  - The value `D` specifies the number of Envelope Clock ticks to wait before the volume is decremented by 1.
  - **Time per step:** `(D + 1) * (1/256 seconds)`

- **F5x2, Bits 7-4: Sustain Level (S)**
  - This 4-bit value (0-15) sets the target volume level for the sustain phase.
  - `0000` (0) is silent, `1111` (15) is maximum volume.
  - The envelope holds at this volume as long as the `KEY_ON` bit is active.

- **F5x2, Bits 3-0: Release Rate (R)**
  - This 4-bit value (0-15) determines how quickly the volume falls from the Sustain Level to 0 after `KEY_ON` is cleared.
  - The value `R` specifies the number of Envelope Clock ticks to wait before the volume is decremented by 1.
  - **Time per step:** `(R + 1) * (1/256 seconds)`

Here is a table showing the approximate time it takes for the volume to change by one step for different rate values:

| Rate Value (A, D, or R) | Ticks per Step | Time per Step (ms) | Total Attack Time (0->15) (ms) |
| :---------------------- | :------------- | :----------------- | :----------------------------- |
| 0                       | 1              | ~3.9               | ~59                            |
| 1                       | 2              | ~7.8               | ~117                           |
| ...                     | ...            | ...                | ...                            |
| 7                       | 8              | ~31.3              | ~469                           |
| ...                     | ...            | ...                | ...                            |
| 15                      | 16             | ~62.5              | ~938                           |

### **Mixer & DSP Register Details**

#### **F580: MIX_CTRL**

| Bit | Name   | Description                                         |
| :-- | :----- | :-------------------------------------------------- |
| 7   | APU_EN | Master APU Enable. 1: On, 0: Off (all sound muted). |
| 6-4 | -      | Unused.                                             |
| 3   | CH3_EN | 1: Channel 3 (Noise) is enabled.                    |
| 2   | CH2_EN | 1: Channel 2 (Wave) is enabled.                     |
| 1   | CH1_EN | 1: Channel 1 (Pulse B) is enabled.                  |
| 0   | CH0_EN | 1: Channel 0 (Pulse A) is enabled.                  |

#### **F584 - F587: PAN0 - PAN3**

Each panning register is one byte and controls a single channel (F584 for CH0, F585 for CH1, etc.).

| Bit | Name  | Description                                         |
| :-- | :---- | :-------------------------------------------------- |
| 7-2 | -     | Unused.                                             |
| 1   | PAN_R | 1: Send this channel's output to the Right speaker. |
| 0   | PAN_L | 1: Send this channel's output to the Left speaker.  |

_(Note: Setting both PAN_L and PAN_R to 1 results in a centered mono signal for that channel.)_

#### **F5A0: DSP_CTRL**

| Bit | Name   | Description                                          |
| :-- | :----- | :--------------------------------------------------- |
| 7   | DSP_EN | Master DSP Enable. 1: On, 0: Off.                    |
| 6-4 | -      | Unused.                                              |
| 3   | CH3_IN | 1: Send Channel 3's output into the DSP echo buffer. |
| 2   | CH2_IN | 1: Send Channel 2's output into the DSP echo buffer. |
| 1   | CH1_IN | 1: Send Channel 1's output into the DSP echo buffer. |
| 0   | CH0_IN | 1: Send Channel 0's output into the DSP echo buffer. |
