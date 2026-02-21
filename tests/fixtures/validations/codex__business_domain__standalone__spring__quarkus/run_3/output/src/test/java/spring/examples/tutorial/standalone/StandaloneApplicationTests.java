package spring.examples.tutorial.standalone;

import static org.junit.jupiter.api.Assertions.assertEquals;

import java.util.logging.Logger;

import jakarta.inject.Inject;
import org.junit.jupiter.api.Test;
import io.quarkus.test.junit.QuarkusTest;
import spring.examples.tutorial.standalone.service.StandaloneService;

@QuarkusTest
class StandaloneApplicationTests {

    private static final Logger logger = Logger.getLogger("standalone.service");

    @Inject
    StandaloneService standaloneService;

    @Test
    void contextLoads() {}

    @Test
    void testReturnMessage() {
        logger.info("Testing standalone.service.StandaloneService.returnMessage()");
        String expResult = "Greetings!";
        String result = standaloneService.returnMessage();
        assertEquals(expResult, result);
    }
}
