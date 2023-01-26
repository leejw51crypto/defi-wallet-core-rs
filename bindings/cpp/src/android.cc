#include <jni.h>
#include <string>
#include "defi-wallet-core-cpp/include/android.h"
#define SECURE_STORAGE_CLASS "com/cronos/play/SecureStorage"
using namespace std;

JNIEnv *g_env = NULL;
namespace org {
namespace defi_wallet_core {

int secureStorageSetJavaEnv(JNIEnv *userenv) {
  g_env = userenv;
  return 1;
}

int secureStorageWriteBasic(JNIEnv *env, string userkey, string uservalue) {

  string secureStorageClass = SECURE_STORAGE_CLASS;
  jclass activityThreadClass = env->FindClass("android/app/ActivityThread");
  jmethodID currentActivityThreadMethod =
      env->GetStaticMethodID(activityThreadClass, "currentActivityThread",
                             "()Landroid/app/ActivityThread;");
  jobject activityThread = env->CallStaticObjectMethod(
      activityThreadClass, currentActivityThreadMethod);
  jmethodID getApplicationMethod = env->GetMethodID(
      activityThreadClass, "getApplication", "()Landroid/app/Application;");
  jobject context = env->CallObjectMethod(activityThread, getApplicationMethod);
  jclass kotlinClass = env->FindClass(secureStorageClass.c_str());
  jmethodID functionMethod = env->GetStaticMethodID(
      kotlinClass, "writeSecureStorage",
      "(Landroid/content/Context;Ljava/lang/String;Ljava/lang/String;)I");
  jstring key = env->NewStringUTF(userkey.c_str());
  jstring value = env->NewStringUTF(uservalue.c_str());
  jint ret = env->CallStaticIntMethod(kotlinClass, functionMethod, context, key,
                                      value);

  return (int)ret;
}

string secureStorageReadBasic(JNIEnv *env, string userkey) {

  string secureStorageClass = SECURE_STORAGE_CLASS;
  jclass activityThreadClass = env->FindClass("android/app/ActivityThread");
  jmethodID currentActivityThreadMethod =
      env->GetStaticMethodID(activityThreadClass, "currentActivityThread",
                             "()Landroid/app/ActivityThread;");
  jobject activityThread = env->CallStaticObjectMethod(
      activityThreadClass, currentActivityThreadMethod);
  jmethodID getApplicationMethod = env->GetMethodID(
      activityThreadClass, "getApplication", "()Landroid/app/Application;");
  jobject context = env->CallObjectMethod(activityThread, getApplicationMethod);
  jclass kotlinClass = env->FindClass(secureStorageClass.c_str());
  jmethodID functionMethod = env->GetStaticMethodID(
      kotlinClass, "readSecureStorage",
      "(Landroid/content/Context;Ljava/lang/String;)Ljava/lang/String;");

  jstring x = env->NewStringUTF(userkey.c_str());
  jobject ret =
      env->CallStaticObjectMethod(kotlinClass, functionMethod, context, x);
  string retstring = string(env->GetStringUTFChars((jstring)ret, 0));

  return retstring;
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
    return rust::String(ret.c_str());
  } catch (exception &e) {
    return rust::String(""); // fail
  }
}

} // namespace defi_wallet_core
} // namespace org