Vue.createApp({
    data() {
        return {
            submissions: [],
            submission: '',
        }
    },

    created() {
        this.fetchSubmissions()
    },

    methods: {
        async fetchSubmissions() {
            try {
                let response = await fetch("/api/votingsubmissions");
                let submissions = await response.json();
                this.submissions = submissions;
            } catch (error) {
                console.error("Error fetching submissions:", error);
            }
        },

        async addSubmission() {
            const description = this.submission && this.submission.trim();
            if (!description) {
                return;
            }

            try {
                await fetch("/api/votingsubmissions/create", {
                    method: "POST",
                    headers: {
                        "Content-Type": "application/json",
                    },
                    body: JSON.stringify({ description }),
                });
                this.submission = '';
                this.fetchSubmissions();
            } catch (error) {
                console.error("Error adding submission:", error);
            }
        },

        async vote(submission) {
            try {
                const response = await fetch("/api/votingsubmissions/" + submission.id + "/vote", {
                    method: "PATCH",
                    headers: {
                        "Content-Type": "application/json",
                    },
                    body: JSON.stringify({}),
                });
                if (response.ok) {
                    submission.vote_count++;
                } else {
                    console.error("Error voting on submission:", response.statusText);
                }
            } catch (error) {
                console.error("Error voting on submission:", error);
            }
        },

        async deleteSubmission(submission) {
            try {
                const response = await fetch("/api/votingsubmissions/" + submission.id, {
                    method: "DELETE",
                });
                if (response.ok) {
                    this.fetchSubmissions();
                } else {
                    console.error("Error deleting submission:", response.statusText);
                }
            } catch (error) {
                console.error("Error deleting submission:", error);
            }
        },
    },
}).mount('.voting-app');
