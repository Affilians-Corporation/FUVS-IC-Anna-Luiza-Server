package main

import (
	"fmt"
	"net/http"
	"time"
)

func sum(seq *[]float64) float64 {
	var sum float64
	for i := range *seq {
		sum += float64(i)
	}
	return sum
}

func main() {
	// var times []float64
	elapsed := time.Since(time.Now())
	for range make([]int, 10) {
		start := time.Now()
		http.Get("http://localhost:8000/theme/test15")
		// if err != nil {
		//	fmt.Println("Could not contact server")
		//	os.Exit(1)
		//}
		elapsed = time.Since(start)
		// times = append(times, float64(elapsed.Milliseconds()))
	}
	fmt.Println("Elapsed: ", elapsed.Milliseconds(), "ms")
	// time.Sleep(5 * time.Second)
	// fmt.Println("Sum: ", sum(&times), "\t\tAvg: ", sum(&times)/float64(len(times)))
}
