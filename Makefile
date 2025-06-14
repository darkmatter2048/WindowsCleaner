# MSYS2 UCRT64环境专用Makefile
CC = gcc
CFLAGS = -Wall -O2
TARGET = WCMain/image_compressor.exe
SRC = $(wildcard c/src/*.c) c/src/stb_impl.c
OBJ = $(SRC:.c=.o)

# MSYS2标准stb路径
INC = -I/ucrt64/include/stb -Ic/include

LIBS = -lm

all: $(TARGET)

$(TARGET): $(OBJ)
	$(CC) $(CFLAGS) $(INC) -o $@ $^ $(LIBS)
	@echo "Build successful. Binary created at $(TARGET)"

%.o: %.c
	$(CC) $(CFLAGS) $(INC) -c $< -o $@

clean:
	rm -f $(OBJ) $(TARGET)

.PHONY: all clean
