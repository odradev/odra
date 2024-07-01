Feature: Increment

    Scenario: S1
        Given empty counter
        Then counter is 0

    Scenario: S2
        Given empty counter
        When counter is incremented by 100
        Then counter is 100

    Scenario: S3
        Given empty counter
        When counter is incremented by 200
        Then counter is 200
