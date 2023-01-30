cp ./app/build/tmp/kotlin-classes/release/com/cronos/play/SecureStorage.class ./make
cd ./make
jar cvf SecureStorage.jar SecureStorage.class

