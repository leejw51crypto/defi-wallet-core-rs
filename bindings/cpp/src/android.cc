#include "rust/cxx.h"
#include <string>
#include <fstream>
#include <iostream>

#define SECURE_STORAGE_CLASS "com/cronos/play/SecureStorage"
using namespace std;

class JNIEnv {
public:
  JNIEnv() {}
  ~JNIEnv() {}
};

JNIEnv *g_env = NULL;

namespace org {
namespace defi_wallet_core {

int secureStorageWriteBasic(JNIEnv *env, string userkey, string uservalue) {

  // open or createfile for overwrite, binary mode
  ofstream file(userkey.c_str(), ios::out | ios::trunc | ios::binary);
  if (!file.is_open()) {
    return 0;
  }
  // convert to uservalue to char array, and write to file
  file.write(uservalue.c_str(), uservalue.length());
  return 1;
}

string secureStorageReadBasic(JNIEnv *env, string userkey) {
  char tmp[1000];

  ifstream file(userkey.c_str(), ios::in | ios::binary);
  if (!file.is_open()) {
    snprintf(tmp, sizeof(tmp),  "{\"result\":\"\",\"success\":\"0\",\"error\":\"encrypt file not found\"}");
    return tmp;
  }

  // read all bytes from file, and convert to string
  file.seekg(0, ios::end);
  int length = file.tellg();
  file.seekg(0, ios::beg);
  char *buffer = new char[length];
  file.read(buffer, length);
  string retstring(buffer, length); 
  snprintf(tmp, sizeof(tmp), "{\"result\":\"%s\",\"success\":\"1\",\"error\":\"\"}",buffer);
  // print tmp
  std::cout<<"returning  =  "<<tmp<<std::endl;
  //std::cout<<"returning  =  "<<tmp<<std::endl;;
  return tmp;
}



int secureStorageWrite(rust::String userkey, rust::String uservalue) {
  try {
    JNIEnv *env = g_env;
    string userkeystring = userkey.c_str();
    string uservaluestring = uservalue.c_str();
    int ret = secureStorageWriteBasic(env, userkeystring, uservaluestring);
    return ret;
  } catch (exception &e) {
    return 0; // fail
  }
}

rust::String secureStorageRead(rust::String userkey) {
  try {
    JNIEnv *env = g_env;
    string ret = secureStorageReadBasic(env, userkey.c_str());
    std::cout<<"read= "<< ret<< std::endl;
    return rust::String(ret.c_str());
  } catch (exception &e) {
    return rust::String(""); // fail
  }
}

} // namespace defi_wallet_core
} // namespace org