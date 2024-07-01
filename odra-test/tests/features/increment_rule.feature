Feature: Increment Rule

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

    Rule: R1

        Background: B1:
            When counter is incremented by 1

        Scenario Outline: R1_OL1
            Then counter is 1
            When counter is incremented by <value>
            Then counter is <result>

            Examples:
                | value | result |
                | 100   | 101    |
                | 200   | 201    |
  