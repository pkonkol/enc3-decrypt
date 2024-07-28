#include <string>
#include <zlib.h>
#include <iostream>
#include <fstream>
#include <sstream>
#include <cstdint>

#define DELTA 0x9e3779b9
// that is also a fun part
// we use all available vars, y,z,sum,p,e,key
#define MX (((z >> 5 ^ y << 2) + (y >> 3 ^ z << 4)) ^ ((sum ^ y) + (key[(p & 3) ^ e] ^ z)))

void bdecrypt(uint8_t *buffer, int len, uint64_t k)
{
    // PREP
    // everything in the mathy loop works on
    uint32_t const key[4] = {(uint32_t)(k >> 32), (uint32_t)(k & 0xFFFFFFFF), 0xDEADDEAD, 0xB00BEEEF};
    uint32_t y, z, sum;
    // pointer to the working array but operating on 4 byte uint values
    uint32_t *v = (uint32_t *)buffer;
    unsigned p, rounds, e;
    // so we don't unscramble anything in small files less than 8 bytes so of len 1 uint32, from 2u32s we go on
    int n = (len - len % 4) / 4; // it's actually the number of accessible uint32's in the v buffer
    if (n < 2)
    {
        return;
    }
    // ALGORITHM
    // is this some known one or dude just figured out his scrambling?
    rounds = 6 + 52 / n; // the longer the file the less rounds are done, this is literally how many times we spin the loop

    sum = rounds * DELTA;
    y = v[0];
    std::cout << "initial values y: " << y << ", sum: " << sum << " ,round: " << rounds << std::endl;
    do
    {
        std::cout << "starting round: " << rounds << ",-------------------- sum: " << sum << std::endl;
        e = (sum >> 2) & 3;
        for (p = n - 1; p > 0; p--) // we scramble all accessible uint32's in each round
        {
            z = v[p - 1];
            auto pmxresult = MX;
            printf("y: %u z:%u sum:%u key:%u,%u,%u,%u p: %u e: %u\n", y, z, sum, key[0], key[1], key[2], key[3], p, e);
            std::cout << "pmxresult: " << p << ":" << pmxresult << ", ";
            return;
            y = v[p] -= pmxresult; // here we assign v[p] a value
        }
        std::cout << "\nafter inner loop z: " << z << " , y: " << y << " ,e: " << e << std::endl;
        z = v[n - 1]; // constantly assign a value from the same index
        // -= & = is right associative so v[0] = v[0] - MX; y = v[0];
        auto mxresult = MX;
        y = v[0] -= mxresult; // for each round we also change v[0] constantly, strange
        sum -= DELTA;         // all we do to sum, drop the constant down to zero in the last round
        std::cout << "mx_result: " << mxresult << " ,v[0] after sub: " << v[0] << " ,y, e, z:" << y << e << z << std::endl;
    } while (--rounds);
}

int m_customEncryption = 0;

bool decryptBuffer(std::string &buffer)
{
    if (buffer.size() < 5)
    {
        return true;
    }

    // check first 4 bytes for magic value
    if (buffer.substr(0, 4).compare("ENC3") != 0)
    {
        return false;
    }

    // grab metadata from the files first 24 bytes
    uint64_t key = *(uint64_t *)&buffer[4];
    uint32_t compressed_size = *(uint32_t *)&buffer[12];
    uint32_t size = *(uint32_t *)&buffer[16];
    uint32_t adler = *(uint32_t *)&buffer[20]; // wtf is adler?

    // guess it's just an error situation?
    // compressed should be smaller so maybe it's just decompressed size actually?
    // ok decompressed size is "size" so compressed_size is what it means. It just has to be exactly what is left in the buffer
    // is there a check for what if that value is larger than a buffer? overflow or shit?
    if (compressed_size < buffer.size() - 24)
    {
        return false;
    }

    auto test2 = buffer.data();
    std::cout << test2 << ";" << std::endl;
    std::cout << "buffer with header b4 scrabmle= [";
    for (int i = 0; i < compressed_size; i++)
    {
        printf("%u", (unsigned char)test2[i]);
        std::cout << ", ";
    }
    std::cout << "]" << std::endl;

    // BDECRYPT
    bdecrypt((uint8_t *)&buffer[24], compressed_size, key);
    // this basically descrambles the bytes of the buffer it seems to retrieve zlib compressed values

    // create new buffer of size stored in metadata
    std::string new_buffer;
    new_buffer.resize(size);
    std::cout << "huj1" << std::endl;
    std::ofstream file;
    file.open("./descrambled-buffer.raw", std::ios_base::binary);
    // assert(file.is_open());
    auto test = buffer.data();
    file.write(test, size);
    // auto testbyte = (const byte *)buffer;

    std::cout << buffer << ";" << std::endl;
    auto test3 = buffer.data();
    // std::cout << test3 << ";" << std::endl;
    std::cout << "buffer w header after scramble c++= [";
    for (int i = 0; i < compressed_size; i++)
    {
        printf("%u", (unsigned char)test3[i]);
        std::cout << ", ";
    }
    std::cout << "]";
    // std::cout << (uint8_t *)&buffer[24] << std::endl;
    // std::cout << (uint8_t *)&buffer[24] << std::endl;
    std::cout << "huj2" << std::endl;
    // why not just use original "size" var tho?
    unsigned long new_buffer_size = new_buffer.size();
    if (
        uncompress(
            (uint8_t *)new_buffer.data(), &new_buffer_size,
            (uint8_t *)&buffer[24], compressed_size) != Z_OK)
    {
        return false;
    }

    // well we could return the new_buffer as well if it was in rust then as we have to allocate it from scratch anyway
    buffer = new_buffer;
    return true;
}

int main()
{
    std::cout << "hello" << std::endl;
    std::string data("");

    std::ifstream t("./init.lua");
    std::stringstream buffer;
    buffer << t.rdbuf();
    data = buffer.str();

    if (decryptBuffer(data))
    {
        std::cout << data << std::endl;
    }
    else
    {
        std::cout << "FAILED" << std::endl;
    }
    return 0;
}
