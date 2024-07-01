Feature: Increment Outline

    Scenario: S1
        Given empty counter
        Then counter is 0

    Scenario Outline: OL1
        Given empty counter
        When counter is incremented by <value>
        Then counter is <value>

        Examples:
            | value |
            | 100   |
            | 200   |

    Scenario Outline: OL2
        Given empty counter
        When counter is incremented by <value>
        When counter is incremented by <value2>
        Then counter is <result>

        Examples:
            | value | value2 | result |
            | 100   | 2      | 102    |
            | 200   | 3      | 203    |
