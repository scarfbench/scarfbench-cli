package spring.examples.tutorial.standalone;

import io.quarkus.runtime.Quarkus;
import io.quarkus.runtime.annotations.QuarkusMain;
import io.quarkus.runtime.annotations.QuarkusApplication;

@QuarkusMain
@QuarkusApplication
public class StandaloneApplication {

    public static void main(String[] args) {
        Quarkus.run(StandaloneApplication.class, args);
    }

    public int run(String... args) throws Exception {
        Quarkus.waitForExit();
        return 0;
    }
}
