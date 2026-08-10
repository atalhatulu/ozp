CC = gcc
CFLAGS = -Wall -Wextra -I./src/include

SRC_DIR = src
OBJ_DIR = obj

TARGET = oz
KONSOL = oz-konsol

SRCS = src/lexer.c src/main.c src/parser.c src/utils.c
OBJS = $(patsubst $(SRC_DIR)/%.c, $(OBJ_DIR)/%.o, $(SRCS))

all: $(TARGET) $(KONSOL)

$(TARGET): $(OBJS)
	$(CC) $(CFLAGS) -o $@ $^

$(KONSOL): $(OBJ_DIR)/konsol.o
	$(CC) $(CFLAGS) -o $@ $^

$(OBJ_DIR)/%.o: $(SRC_DIR)/%.c | $(OBJ_DIR)
	$(CC) $(CFLAGS) -c $< -o $@

$(OBJ_DIR):
	mkdir -p $(OBJ_DIR)

clean:
	rm -rf $(OBJ_DIR) $(TARGET) program gecici_oz_cikti.c

.PHONY: all clean
