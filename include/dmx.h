#ifndef DMX_H
#define DMX_H

#include <stdint.h>

/**
 * @brief Opens a DMX serial port for communication
 *
 * Opens the specified serial port and configures it for DMX512 communication
 * (250000 baud, 8 data bits, no parity, 2 stop bits).
 *
 * @param port The name of the serial port (e.g., "COM3" on Windows,
 * "/dev/ttyUSB0" on Linux)
 * @return File descriptor/handle on success (>= 0), -1 on error
 */
int dmx_open(const char* port);

/**
 * @brief Sends a DMX BREAK signal
 *
 * Generates the DMX BREAK condition required before transmitting DMX data.
 * This signals the start of a new DMX frame to all connected devices.
 *
 * @param fd File descriptor returned by dmx_open()
 */
void dmx_send_break(int fd);

/**
 * @brief Writes DMX data to the serial port
 *
 * Transmits DMX channel data. Typically called after dmx_send_break().
 * The data should start with a start code (usually 0x00) followed by
 * up to 512 channel values.
 *
 * @param fd File descriptor returned by dmx_open()
 * @param data Pointer to the DMX data buffer
 * @param length Number of bytes to write (max 513: start code + 512 channels)
 * @return Number of bytes written on success, -1 on error
 */
int dmx_write(int fd, const uint8_t* data, int length);

/**
 * @brief Reads a DMX frame from the serial port
 *
 * Attempts to read incoming DMX data. This function may return immediately
 * if no data is available (non-blocking).
 *
 * @param fd File descriptor returned by dmx_open()
 * @param buffer Buffer to store the received DMX data
 * @param max_len Maximum number of bytes to read (buffer size)
 * @return Number of bytes read (0 if no data available), -1 on error
 */
int dmx_read_frame(int fd, uint8_t* buffer, int max_len);

/**
 * @brief Closes the DMX port
 *
 * Closes the serial port and releases all associated resources.
 * The file descriptor becomes invalid after this call.
 *
 * @param fd File descriptor returned by dmx_open()
 */
void dmx_close(int fd);

#endif  // DMX_H