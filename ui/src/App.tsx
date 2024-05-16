import { useState } from "react";
import {
  Alert,
  Button,
  CircularProgress,
  CssBaseline,
  Snackbar,
  Stack,
  TextField,
  Typography,
  useTheme,
} from "@mui/material";

function App() {
  const theme = useTheme();
  const [password, setPassword] = useState<string | null>(null);
  const [loading, setLoading] = useState<boolean>(false);

  const [alertOpen, setAlertOpen] = useState<boolean>(false);
  const [alertMessage, setAlertMessage] = useState<string | null>(null);
  const [alertIsError, setAlertIsError] = useState<boolean>(false);

  const sendRestartRequest = async () => {
    setLoading(true);

    const response = await fetch("/minecraft/api/restart", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ password }),
    });

    setLoading(false);

    if (response.status === 200) {
      setAlertIsError(false);
      setAlertMessage("Server successfully restarted");
      setAlertOpen(true);
    } else {
      setAlertIsError(true);
      setAlertMessage("Error restarting server");
      setAlertOpen(true);
    }
  };

  const onAlertClose = () => {
    setAlertOpen(false);
  };

  return (
    <>
      <CssBaseline />

      <Stack direction="column" spacing={2} padding={theme.spacing(2)}>
        <Typography variant="h4">Minecraft Control Panel</Typography>

        <Stack direction="row" spacing={1} alignItems="center">
          <TextField
            type="password"
            label="Password"
            value={password}
            onChange={(event: React.ChangeEvent<HTMLInputElement>) => {
              setPassword(event.target.value);
            }}
          />
          <Button
            variant="contained"
            disabled={loading || password === null || password.trim() === ""}
            onClick={() => sendRestartRequest()}
          >
            Restart
          </Button>

          {loading && <CircularProgress variant="indeterminate" />}
        </Stack>
      </Stack>

      <Snackbar
        open={alertOpen}
        autoHideDuration={600000000}
        onClose={onAlertClose}
      >
        <Alert
          onClose={onAlertClose}
          severity={alertIsError ? "error" : "success"}
          variant="filled"
          sx={{ width: "100%" }}
        >
          {alertMessage}
        </Alert>
      </Snackbar>
    </>
  );
}

export default App;
