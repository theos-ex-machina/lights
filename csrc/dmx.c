#ifdef _WIN32
#include <stdint.h>
#include <windows.h>

// Windows implementation
int dmx_open(const char* port) {
  HANDLE handle = CreateFileA(port, GENERIC_READ | GENERIC_WRITE, 0, NULL,
                              OPEN_EXISTING, 0, NULL);

  if (handle == INVALID_HANDLE_VALUE) {
    return -1;
  }

  // Configure serial port for DMX (250000 baud, 8N2)
  DCB dcb = {0};
  dcb.DCBlength = sizeof(DCB);

  if (!GetCommState(handle, &dcb)) {
    CloseHandle(handle);
    return -1;
  }

  dcb.BaudRate = 250000;
  dcb.ByteSize = 8;
  dcb.Parity = NOPARITY;
  dcb.StopBits = TWOSTOPBITS;

  if (!SetCommState(handle, &dcb)) {
    CloseHandle(handle);
    return -1;
  }

  // Return handle as integer (this is a simplification)
  return (int)(intptr_t)handle;
}

void dmx_send_break(int fd) {
  HANDLE handle = (HANDLE)(intptr_t)fd;

  // Send BREAK condition (minimum 88μs, we use 100μs)
  SetCommBreak(handle);
  Sleep(1);  // 1ms = 1000μs (much longer than minimum, but safe)
  ClearCommBreak(handle);

  // Send MAB (Mark After Break) - minimum 8μs, we use 12μs
  // Line is already in MARK state after ClearCommBreak
  // Windows Sleep() minimum is 1ms, so we can't get precise μs timing
  // For better timing, you'd need QueryPerformanceCounter or multimedia timers
  // For now, the 1ms break includes adequate MAB time
}

int dmx_read_frame(int fd, uint8_t* buffer, int length) {
  HANDLE handle = (HANDLE)(intptr_t)fd;
  DWORD bytes_read;

  if (!ReadFile(handle, buffer, length, &bytes_read, NULL)) {
    return -1;
  }

  return (int)bytes_read;
}

int dmx_write(int fd, const uint8_t* data, int length) {
  HANDLE handle = (HANDLE)(intptr_t)fd;
  DWORD bytes_written;

  if (!WriteFile(handle, data, length, &bytes_written, NULL)) {
    return -1;
  }

  return (int)bytes_written;
}

void dmx_close(int fd) {
  HANDLE handle = (HANDLE)(intptr_t)fd;
  CloseHandle(handle);
}

#else
// Unix/Linux implementation
#include <fcntl.h>
#include <stdint.h>
#include <termios.h>
#include <unistd.h>

int dmx_open(const char* port) {
  int fd = open(port, O_RDWR | O_NOCTTY | O_NONBLOCK);
  if (fd < 0) return -1;

  struct termios options;
  tcgetattr(fd, &options);
  cfsetispeed(&options, B250000);
  cfsetospeed(&options, B250000);
  options.c_cflag |= CS8 | CSTOPB;  // 8 data bits, 2 stop bits
  tcsetattr(fd, TCSANOW, &options);

  return fd;
}

void dmx_send_break(int fd) {
  // Send BREAK condition
  tcsendbreak(fd, 0);  // duration 0 = default break time (usually 250-500ms)

  // Send MAB (Mark After Break) - minimum 8μs, we use 12μs
  usleep(12);  // 12 microseconds MAB

  // Note: tcsendbreak() timing varies by system
  // For precise DMX timing, you might need custom break generation
}

int dmx_write(int fd, const uint8_t* data, int length) {
  return write(fd, data, length);
}

int dmx_read_frame(int fd, uint8_t* buffer, int max_len) {
  int n = read(fd, buffer, max_len);
  if (n < 0) return -1;
  return n;
}

void dmx_close(int fd) { close(fd); }

#endif