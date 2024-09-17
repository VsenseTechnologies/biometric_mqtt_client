#!/bin/bash

HOST="34.47.150.239"
PORT=1883
TOPIC="vs24al001/attendence"
COUNT=300  
INTERVAL=0.1  

for (( i=1; i<=COUNT; i++ ))
do
    MESSAGE_PAYLOAD='{"unit_id":"vs24al001","student_id":"4","time_stamp":"10:30"}'
    mosquitto_pub -h $HOST -p $PORT  -t $TOPIC -m "$MESSAGE_PAYLOAD" &
    sleep $INTERVAL
done

wait

