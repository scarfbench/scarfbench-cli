package spring.examples.tutorial.standalone;

import static org.junit.jupiter.api.Assertions.assertEquals;
import java.util.logging.Logger;
import org.junit.jupiter.api.Test;
import io.quarkus.test.junit.QuarkusTest;
import jakarta.inject.Inject;
import spring.examples.tutorial.standalone.service.StandaloneService;

@QuarkusTest
class StandaloneApplicationTests {

    private static final Logger logger = Logger.getLogger("standalone.service");

    @Inject
    StandaloneService standaloneService;

    @Test
    void contextLoads() {}

    @Test
    public void testReturnMessage() throws Exception {
        logger.info("Testing standalone.service.StandaloneService.returnMessage()");
        String expResult = "Greetings!";
        String result = standaloneService.returnMessage();
        assertEquals(expResult, result);
    }

}
