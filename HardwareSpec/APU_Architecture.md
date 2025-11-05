# **Cicada-16 APU - Architecture**

This document describes the design of the Audio Processing Unit (APU) for the Cicada-16 fantasy console. The APU is responsible for generating all sound and music. The design is inspired by classic sound chips from the 8-bit and 16-bit eras, featuring four independent sound channels with ADSR envelope control and a simple DSP for effects.

## **1. APU Registers (F080–F0BF)**

This section details the memory-mapped registers used to control all APU functions, starting with a high-level overview.

### **Register Map Overview**

| Address   | Name         | Description                                                |
| :-------- | :----------- | :--------------------------------------------------------- |
| F080      | **reserved** | Reserved for future use                                    |
| F081      | CH0_CTRL     | Pulse A: Control Register                                  |
| F082-F083 | CH0_ADSR     | ADSR settings for Channel 0                                |
| F084-F085 | CH0_FREQ     | 16-bit frequency for Channel 0                             |
| F086      | CH0_SWP      | Sweep settings for Channel 0                               |
| F087      | CH1_CTRL     | Pulse B: Control Register                                  |
| F088-F089 | CH1_ADSR     | ADSR settings for Channel 1                                |
| F08A-F08B | CH1_FREQ     | 16-bit frequency for Channel 1                             |
| F08C      | **reserved** | Reserved for future use                                    |
| F08D      | CH2_CTRL     | Wave channel: Control Register                             |
| F08E-F08F | CH2_ADSR     | ADSR settings for Channel 2                                |
| F090-F091 | CH2_FREQ     | 16-bit frequency for Channel 2                             |
| F092      | **reserved** | Reserved for future use                                    |
| F093      | CH3_CTRL     | Noise channel: Control Register                            |
| F094-F095 | CH3_ADSR     | ADSR settings for Channel 3                                |
| F096      | MIX_CTRL     | Master APU enable, individual channel enables              |
| F097      | MIX_VOL_L    | Master volume Left (0-15)                                  |
| F098      | MIX_VOL_R    | Master volume Right (0-15)                                 |
| F099      | CH0_OUT      | Pulse A channel stereo volume/panning control              |
| F09A      | CH1_OUT      | Pulse B channel stereo volume/panning control              |
| F09B      | CH2_OUT      | Wave channel stereo volume/panning control                 |
| F09C      | CH3_OUT      | Noise channel stereo volume/panning control                |
| F09D      | DSP_CTRL     | DSP Control Register                                       |
| F09E      | DSP_DELAY    | Delay time/length (controls read offset into delay buffer) |
| F09F      | DSP_FBACK    | Feedback level (0-15). Controls echo decay.                |
| F0A0      | DSP_WET      | Wet signal mix level (0-15). Controls echo volume.         |
| F0A1-F0BF | **reserved** | Reserved for future use                                    |

## **2. Core Features**

- **4 Independent Channels**: The APU can play four sounds simultaneously.
- **Channel 0 (Pulse A)**: A square wave generator with variable duty cycle and a hardware frequency sweep unit.
- **Channel 1 (Pulse B)**: A second square wave generator, identical to Pulse A but without the sweep unit.
- **Channel 2 (Wave)**: A flexible waveform generator that plays back a custom 32-sample waveform from Wave RAM.
- **Channel 3 (Noise)**: A noise generator capable of producing various "colors" of noise for percussion and SFX.
- **ADSR Envelopes**: The Pulse, Wave, and Noise channels each feature a unified, hardware-based 4-stage ADSR volume envelope.
- **DSP (Digital Signal Processor)**: A simple hardware unit for creating an echo/delay effect.
- **Stereo Panning**: Each channel's output can be independently panned.
- **Master Volume Control**: Global volume control for both left and right stereo outputs.

## **3. Channel 0 & 1: Pulse Channels**

The two pulse channels generate simple square waves. They are controlled by a set of registers that define their pitch, volume, and timbre.

- **Frequency Control**: A 16-bit frequency register allows for a wide range of musical notes.
- **Volume Envelope**: A hardware ADSR envelope generator for shaping the volume of notes.
- **Duty Cycle**: The "width" of the square wave can be changed (12.5%, 25%, 50%, 75%) to alter the timbre.
- **Frequency Sweep (Channel 0 Only)**: A hardware unit that can automatically slide the frequency of Channel 0 up or down over time.

### 3.1. Frequency and Pitch

The pitch of the two pulse channels is determined by the 16-bit value in their respective `CHx_FREQ` registers (F084-F085 for CH0, F089-F08A for CH1). The APU uses a master clock of **2.097152 MHz**. The output frequency is calculated with the following formula:

**`Output Frequency (Hz) = 2,097,152 / (64 * (65536 - FREQ_REG))`**

Where `FREQ_REG` is the 16-bit value from the frequency register. This formula allows for fine-grained control over the pitch, covering a wide range of musical notes.

**Example Calculation:**

To produce the note A-4 (440 Hz), a programmer would need to calculate the required `FREQ_REG` value:

1.  `440 = 2,097,152 / (64 * (65536 - FREQ_REG))`
2.  `64 * (65536 - FREQ_REG) = 2,097,152 / 440`
3.  `65536 - FREQ_REG = 4766.25 / 64`
4.  `65536 - FREQ_REG = 74.47...`
5.  `FREQ_REG = 65536 - 74` (rounding to the nearest integer)
6.  `FREQ_REG = 65462`

A value of **65462** in the frequency register will produce a tone very close to 440 Hz. Game developers would typically use a pre-calculated lookup table for note frequencies. The official System Library provides a standardized lookup table for this purpose. See the `System_Library.md` document for more details.

### 3.2. Hardware Frequency Sweep (Channel 0)

Channel 0 includes a hardware frequency sweep unit that can be used to create sliding pitch effects, such as arpeggios, explosions, and risers. The sweep unit periodically recalculates the channel's frequency based on its current frequency and the settings in the `CH0_SWP` register.

The sweep unit is clocked at **128Hz**. The timing of the sweep is controlled by the `SWP_TIME` setting. When the sweep is enabled, the following calculation is performed at each step:

**`New Frequency = Current Frequency +/- (Current Frequency >> SWP_SHIFT)`**

The direction of the sweep (addition or subtraction) is controlled by the `SWP_DIR` bit. If the new calculated frequency goes out of the valid range (0-65535), the channel is disabled.

The sweep unit is controlled by the `CH0_SWP` register.

### 3.3. Pulse Channel Registers

#### **F081: CH0_CTRL (Pulse A)**

| Bit | Name   | Description                                                                                             |
| :-- | :----- | :------------------------------------------------------------------------------------------------------ |
| 7   | KEY_ON | 1 = Note On (Attack->Decay->Sustain). 0 = Note Off (Release). The channel is active when this bit is 1. |
| 6-5 | DUTY   | Square wave duty cycle: 00: 12.5%, 01: 25%, 10: 50%, 11: 75%                                            |
| 4-0 | -      | Unused.                                                                                                 |

#### **F086: CH0_SWP (Pulse A Sweep)**

| Bit | Name      | Description                                                                                                                                                          |
| :-- | :-------- | :------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 7   | SWP_EN    | 1 = Enable the frequency sweep unit. The sweep is triggered when a note is keyed on.                                                                                 |
| 6-4 | SWP_TIME  | Time between sweep steps, in units of 7.8ms (1/128s). If 0, sweep is disabled. (Range: 0-7)                                                                          |
| 3   | SWP_DIR   | Sweep direction. 0: Addition (frequency increases), 1: Subtraction (frequency decreases).                                                                            |
| 2-0 | SWP_SHIFT | Sweep magnitude. The new frequency is calculated by shifting the current frequency this many bits to the right and adding/subtracting it. If 0, sweep has no effect. |

#### **F084-F085: CH0_FREQ**

This 16-bit register controls the frequency of the pulse wave for Channel 0.

#### **F087: CH1_CTRL (Pulse B)**

| Bit | Name   | Description                                                                                             |
| :-- | :----- | :------------------------------------------------------------------------------------------------------ |
| 7   | KEY_ON | 1 = Note On (Attack->Decay->Sustain). 0 = Note Off (Release). The channel is active when this bit is 1. |
| 6-5 | DUTY   | Square wave duty cycle: 00: 12.5%, 01: 25%, 10: 50%, 11: 75%                                            |
| 4-0 | -      | Unused.                                                                                                 |

#### **F08A-F08B: CH1_FREQ**

This 16-bit register controls the frequency of the pulse wave for Channel 1.

## **4. Channel 2: Wave Channel**

The wave channel offers the most sonic flexibility by playing back a small, user-defined waveform. This is ideal for creating unique instrument tones or simple speech/sound effects.

- **Wave RAM:** A 1 KiB (1024-byte) area of memory at `FA00-FDFF` is dedicated to waveform data. This RAM is organized as a bank of 32 unique waveforms, indexed from 0 to 31.
- **Waveform Format:** Each waveform is 32 bytes long and contains 64 sequential 4-bit samples. The samples are packed two per byte, with the high nibble (bits 7-4) being the first sample and the low nibble (bits 3-0) being the second. The 4-bit sample values represent amplitude, from 0 (lowest point) to 15 (highest point).
- **Playback:** When the channel is triggered via `KEY_ON`, the APU begins reading the 64 samples of the selected waveform sequentially. The playback speed is controlled by the `CH2_FREQ` register. When the 64th sample is played, the playback loops back to the first sample. The output of the wave channel is the current sample's 4-bit value, scaled by the channel's current 4-bit ADSR envelope volume.
- **Volume Envelope:** This channel includes the same hardware ADSR envelope found on the Pulse and Noise channels.

### 4.1. Waveform Frequency

The frequency of the `CH2_FREQ` register determines how quickly the APU steps through the 64 samples of the selected waveform. The resulting audible pitch is determined by this playback rate. The formula is intentionally aligned with the pulse channels, allowing them to share the same frequency lookup table.

**`Sample Rate (Hz) = 2,097,152 / (65536 - FREQ_REG)`**

**`Output Pitch (Hz) = Sample Rate / 64`**

This simplifies to the final formula:

**`Output Pitch (Hz) = 2,097,152 / (64 * (65536 - FREQ_REG))`**

Where `FREQ_REG` is the 16-bit value from the `CH2_FREQ` register (F08E-F08F).

**Note:** Because this formula is identical to the one for the pulse channels, the same note frequency values from the System Library's lookup table can be used for Channel 2.

**Example Calculation:**

To produce the note A-4 (440 Hz), the `FREQ_REG` value is the same as for the pulse channels:

`FREQ_REG = 65462`

This value will cause the 64 samples of the waveform to be played back at a rate that produces a fundamental pitch of 440 Hz.

### 4.2. Wave Channel Registers

#### **F08D: CH2_CTRL (Wave)**

| Bit | Name     | Description                                                                                             |
| :-- | :------- | :------------------------------------------------------------------------------------------------------ |
| 7   | KEY_ON   | 1 = Note On (Attack->Decay->Sustain). 0 = Note Off (Release). The channel is active when this bit is 1. |
| 6   | -        | Unused.                                                                                                 |
| 5-0 | WAVE_IDX | Selects which 32-byte waveform to play from Wave RAM (0-31).                                            |

#### **F090-F091: CH2_FREQ**

This 16-bit register controls the playback frequency of the selected waveform for Channel 2.

## **5. Channel 3: Noise Channel**

The noise channel generates pseudo-random noise, which is essential for creating percussive sounds (drums, cymbals) and sound effects (explosions, static). Its sound is generated by a Linear Feedback Shift Register (LFSR) that is clocked at a variable rate.

### 5.1. LFSR Implementation

The LFSR is a shift register whose input bit is derived from the exclusive-OR (XOR) of two bits in its own state. At each clock tick, all bits are shifted one position to the right, and the new input bit becomes the most significant bit (MSB). The output of the channel is determined by the least significant bit (LSB).

The `LFSR_MODE` bit in the `CH3_CTRL` register selects between two different LFSR configurations, which produce different timbres:

- **Mode 0 (15-bit LFSR):**
  - Creates a full, dense "white noise" sound.
  - The new input bit is the result of `Bit 14 XOR Bit 15`.
- **Mode 1 (7-bit LFSR):**
  - Creates a metallic, periodic sound because the sequence of pseudo-random numbers is much shorter.
  - The new input bit is the result of `Bit 6 XOR Bit 7`.

The channel's audio output is `1` if the LSB of the LFSR is `0`, and `-1` if the LSB is `1`. This output is then scaled by the channel's current ADSR envelope volume.

### 5.2. Noise Frequency

The "pitch" of the noise is controlled by the rate at which the LFSR is clocked. This clock rate is derived from the master APU clock and the 6-bit `CLK_DIV` value in the `CH3_CTRL` register.

**`LFSR Clock Rate (Hz) = 2,097,152 / (256 * (CLK_DIV + 1))`**

- A lower `CLK_DIV` value results in a higher-pitched, "brighter" noise.
- A higher `CLK_DIV` value results in a lower-pitched, "rumbling" noise.

### 5.3. Noise Channel Registers

#### **F093: CH3_CTRL (Noise)**

| Bit | Name      | Description                                                                                             |
| :-- | :-------- | :------------------------------------------------------------------------------------------------------ |
| 7   | KEY_ON    | 1 = Note On (Attack->Decay->Sustain). 0 = Note Off (Release). The channel is active when this bit is 1. |
| 6   | LFSR_MODE | 0: 15-bit LFSR (White noise), 1: 7-bit LFSR (Metallic noise).                                           |
| 5-0 | CLK_DIV   | Clock divider for the LFSR, controls the base pitch of the noise.                                       |

## **6. ADSR Volume Envelopes**

The APU uses a traditional 4-stage ADSR (Attack, Decay, Sustain, Release) envelope to control channel volume. The envelope's volume level is a 4-bit value (0-15), which modulates the channel's output. The envelope's state machine is updated by a global **Envelope Clock** running at **256Hz**.

When a channel's `KEY_ON` bit is set to 1, the envelope enters the **Attack** phase, where the volume rises from 0 to its peak (15). When it reaches the peak, it moves to the **Decay** phase, where it falls to the specified **Sustain** level. It then remains in the **Sustain** phase as long as `KEY_ON` is 1. When `KEY_ON` is cleared to 0, it immediately enters the **Release** phase, where the volume falls from the current level to 0.

### 6.1. ADSR Registers

Each channel has a pair of registers to control its ADSR envelope: `CH0_ADSR` (F082-F083), `CH1_ADSR` (F088-F089), `CH2_ADSR` (F08E-F08F), and `CH3_ADSR` (F094-F095).

- **First byte, Bits 7-4: Attack Rate (A)**

  - This 4-bit value (0-15) determines how quickly the volume rises from 0 to its peak of 15.
  - The value `A` specifies the number of Envelope Clock ticks to wait before the volume is incremented by 1.
  - **Time per step:** `(A + 1) * (1/256 seconds)`
  - **Total Attack Time (0 to 15):** `15 * (A + 1) * ~3.9ms`

- **First byte, Bits 3-0: Decay Rate (D)**

  - This 4-bit value (0-15) determines how quickly the volume falls from its peak of 15 to the Sustain Level.
  - The value `D` specifies the number of Envelope Clock ticks to wait before the volume is decremented by 1.
  - **Time per step:** `(D + 1) * (1/256 seconds)`

- **Second byte, Bits 7-4: Sustain Level (S)**

  - This 4-bit value (0-15) sets the target volume level for the sustain phase.
  - `0000` (0) is silent, `1111` (15) is maximum volume.
  - The envelope holds at this volume as long as the `KEY_ON` bit is active.

- **Second byte, Bits 3-0: Release Rate (R)**
  - This 4-bit value (0-15) determines how quickly the volume falls from the Sustain Level to 0 after `KEY_ON` is cleared.
  - The value `R` specifies the number of Envelope Clock ticks to wait before the volume is decremented by 1.
  - **Time per step:** `(R + 1) * (1/256 seconds)`

## **7. DSP (Digital Signal Processor)**

The APU includes a simple DSP unit that applies a single echo/delay effect to the audio. This effect is also often called a "delay line".

### 7.1. DSP Signal Flow

The DSP operates on the final mixed audio signal just before it is sent to the speakers. The process works as follows:

1.  **Input Mix:** The audio from any channel with its corresponding `CHx_IN` bit set in `DSP_CTRL` is summed together. This becomes the "dry" input signal for the DSP.
2.  **Delay Line:** The dry signal is written to a circular buffer in the **Delay Buffer RAM** (`F600-F9FF`). A "read head" retrieves a sample from an earlier point in the buffer. The distance between the read and write heads is the delay time.
3.  **Feedback:** A portion of the retrieved (delayed) signal, scaled by the `DSP_FBACK` register, is added back into the dry input signal before it's written to the delay buffer. This creates repeating, decaying echoes.
4.  **Final Mix:** The retrieved (delayed) signal is scaled by the `DSP_WET` register and added to the original, unprocessed dry signal. This final mix is what is sent to the main stereo panning and volume controls.

### 7.2. Delay Buffer and Timing

- **Sample Rate:** The DSP, and the entire APU output mixer, operates at a fixed sample rate of **32,768 Hz**.
- **Delay Buffer:** The buffer is 1024 bytes long, allowing for 1024 8-bit signed audio samples.
- **Delay Time:** The `DSP_DELAY` register controls the delay time. The delay is measured in steps of 4 samples.
  - **`Delay (samples) = (DSP_DELAY + 1) * 4`**
  - **`Delay (seconds) = Delay (samples) / 32768`**
  - This allows for a delay time ranging from ~0.12ms to ~31.25ms. This is short, but useful for creating chorus, flange, or simple reverb-like spatial effects.

### 7.3. DSP Registers

#### **F09D: DSP_CTRL**

| Bit | Name   | Description                                          |
| :-- | :----- | :--------------------------------------------------- |
| 7   | DSP_EN | Master DSP Enable. 1: On, 0: Off.                    |
| 6-4 | -      | Unused.                                              |
| 3   | CH3_IN | 1: Send Channel 3's output into the DSP echo buffer. |
| 2   | CH2_IN | 1: Send Channel 2's output into the DSP echo buffer. |
| 1   | CH1_IN | 1: Send Channel 1's output into the DSP echo buffer. |
| 0   | CH0_IN | 1: Send Channel 0's output into the DSP echo buffer. |

#### **F09E: DSP_DELAY**

An 8-bit register that controls the delay time by setting the read offset into the delay buffer. The delay is `(value + 1) * 4` samples.

#### **F09F: DSP_FBACK**

A 4-bit value (0-15) that controls the feedback level (how much of the delayed signal is mixed back into the delay line input).

- `0`: No feedback, only a single echo is produced.
- `15`: Maximum feedback, creating long, repeating echoes.
- **Formula:** `Feedback = DelayedSample * (DSP_FBACK / 16)`

#### **F0A0: DSP_WET**

A 4-bit value (0-15) that controls the wet/dry mix. This is the volume of the delayed signal in the final output.

- `0`: No echo is heard (fully dry signal).
- `15`: Echo is at its loudest (wet signal).
- **Formula:** `Output = DrySignal + (DelayedSignal * (DSP_WET / 16))`

## 8. APU Signal and Clock Flow

This section provides a detailed, step-by-step overview of the entire APU signal path, from individual channel generation to the final mixed output.

### 8.1. The Clock Tree

The APU uses several different clocks, all derived from the master APU clock, to control different aspects of sound generation.

- **Master APU Clock (2.097152 MHz):** The high-speed master clock that drives all APU operations. Its frequency is exactly one-eighth of the main system clock (16.777216 MHz / 8). It is divided down to create the other, slower clocks.
- **Generator Clocks (Variable):** This is the clock that drives the core sound generators (the pulse wave, wave table, and LFSR). Its speed is controlled by the `FREQ` or `CLK_DIV` registers for each channel and determines the fundamental pitch or character of the raw sound.
- **Envelope Clock (256 Hz):** A slow, fixed-rate clock. 256 times per second, it tells the ADSR unit on each channel whether it should increment or decrement its volume according to its current phase (Attack, Decay, or Release).
- **Sweep Clock (128 Hz):** A slow, fixed-rate clock. 128 times per second, it tells the sweep unit on Channel 0 to perform its frequency recalculation.
- **Mixer Sample Rate (32,768 Hz):** This is the "heartbeat" of the entire mixing system. At every one of these 32,768 ticks per second, the APU performs the full mixing process described below.

### 8.2. The Signal Flow: A Sample's Journey

The following sequence of events happens 32,768 times per second to produce one final audio sample for the left and right speakers.

#### **Step 1: Raw Signal Generation**

For each of the four channels, the APU determines its raw output for the current cycle.

- The channel's core generator (pulse, wave, or noise) produces its signal (e.g., a high/low state, a 4-bit sample from RAM, or a 1/0 from the LFSR).
- The channel's ADSR unit provides its current 4-bit volume level (0-15).
- The raw signal is scaled by this envelope volume. This produces the channel's "pre-mix" output signal.

#### **Step 2: DSP Send (The "Dry" Signal)**

The APU determines what to send to the DSP's echo/delay effect.

- It checks the `DSP_CTRL` register.
- It takes the "pre-mix" output from every channel whose corresponding `CHx_IN` bit is set to `1`.
- These signals are summed together. This combined signal is the "dry" input for the DSP.

#### **Step 3: The DSP Delay Line**

The echo effect is processed.

1.  **Read:** The DSP reads an old sample from its 1KB circular delay buffer. The read position is determined by the `DSP_DELAY` register. This retrieved sample is the "wet" signal (the echo).
2.  **Feedback:** This "wet" signal is scaled down by the `DSP_FBACK` value.
3.  **Write:** The scaled-down feedback signal is added to the "dry" signal from Step 2, and the result is written back into the delay buffer at the current write position.

#### **Step 4: Final Channel Mixing (Volume & Panning)**

This is where the `CHx_OUT` registers are used. The APU calculates the final mix for the left and right speakers, starting with two empty accumulators, `FinalLeft` and `FinalRight`.

For each of the four channels:

1.  It takes the channel's "pre-mix" output from Step 1.
2.  It looks at the channel's `CHx_OUT` register (`F099` - `F09C`).
3.  The pre-mix signal is scaled by the `VOL_L` nibble (0-15) and added to the `FinalLeft` accumulator.
4.  The pre-mix signal is scaled by the `VOL_R` nibble (0-15) and added to the `FinalRight` accumulator.

After this process is complete for all four channels, `FinalLeft` and `FinalRight` contain the complete "dry" mix, perfectly panned and with individual channel volumes applied.

#### **Step 5: DSP Return (The "Wet" Signal)**

The "wet" signal (the echo read from the delay buffer in Step 3) is now added to the final mix.

- The wet signal is scaled by the `DSP_WET` register value.
- This scaled echo is added to _both_ the `FinalLeft` and `FinalRight` accumulators.

#### **Step 6: Master Volume & Output**

This is the final stage before the sound is sent to the speakers.

- The `FinalLeft` signal is scaled by the master left volume (`MIX_VOL_L`).
- The `FinalRight` signal is scaled by the master right volume (`MIX_VOL_R`).
- These two resulting values are the final samples sent to the digital-to-analog converter (DAC).

---

© 2025 Connor Nolan. This work is licensed under a
[Creative Commons Attribution-ShareAlike 4.0 International License](http://creativecommons.org/licenses/by-sa/4.0/).
